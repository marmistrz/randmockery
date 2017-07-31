extern crate nix;
extern crate rand;
#[macro_use]
extern crate enum_extract;
extern crate libloading;
#[macro_use]
extern crate clap;

use std::process::Command;
use nix::sys::signal::Signal;
use nix::sys::wait::{wait, WaitStatus};
use nix::unistd::Pid;

mod ptrace_mod;
pub mod syscall_override;
pub mod args;

use syscall_override::{OverrideRegistry, HandlerData, OverrideData};

pub fn parse_args() -> Vec<String> {
    use std::env;

    let mut args_it = env::args();
    let executable = args_it.next().unwrap();
    let command: Vec<_> = args_it.collect();

    if command.len() == 0 {
        println!("Usage: {} command", executable);
        std::process::exit(1);
    }

    command
}

pub fn spawn_child(mut command: Command) -> Pid {
    use ptrace_mod::PtraceSpawnable;

    let child = command.spawn_ptrace().expect(
        "Error spawning the child process",
    );

    Pid::from_raw(child.id() as i32) // This is awful, see https://github.com/nix-rust/nix/issues/656
}

macro_rules! wait_sigtrap {
    () => {
        match wait() {
            // TODO use PTRACE_O_TRACESYSGOOD
            // See this pull request: https://github.com/nix-rust/nix/pull/566
            Ok(WaitStatus::Exited(_, code)) => {
                println!("Inferior quit with code {}!", code);
                return code;
            }
            Ok(WaitStatus::Stopped(pid, Signal::SIGTRAP)) => pid,
            Ok(s) => panic!("Unexpected stop reason: {:?}", s),
            Err(e) => panic!("Unexpected waitpid error: {:?}", e),
        }
    }
}

pub fn ptrace_setmem<F>(pid: Pid, data: &HandlerData, gen: &mut F)
where
    F: FnMut() -> u8,
{
    use std::mem;

    let step = mem::size_of::<usize>();

    let_extract!(
        HandlerData::Buffer {
            bufptr: ptr,
            buflen: len,
        },
        *data,
        panic!("Mismatched HandlerData variant")
    );

    let end = ptr + len;
    let mut curr = ptr;
    let mut next = curr + step;

    {
        let mut genword = || -> u64 {
            let mut word: [u8; 8] = [0; 8];
            for x in word.iter_mut() {
                *x = gen();
            }

            unsafe { mem::transmute(word) }
        };

        while next < end {
            ptrace_mod::pokedata(pid, curr, genword()).expect(
                "Error changing the child process memory",
            );
            curr += step;
            next += step;
        }
    }


    let lastword = ptrace_mod::peekdata(pid, curr).expect("Error peeking the child process memory");
    let numzero = end - curr;
    let newword: u64;

    unsafe {
        let mut bytes: [u8; 8] = mem::transmute(lastword);
        for i in 0..numzero {
            bytes[i] = gen();
        }
        newword = mem::transmute(bytes);
    }

    ptrace_mod::pokedata(pid, curr, newword).expect(
        "Error changing the child process memory (last, incomplete bytes)",
    );
}

/// Return value: exitcode
pub fn intercept_syscalls(root_pid: Pid, mut reg: OverrideRegistry) -> i8 {
    use std::collections::HashMap;
    let mut map: HashMap<Pid, Option<OverrideData>> = HashMap::new();

    // TODO waitpid here?
    let initial_pid = wait_sigtrap!(); // there will be an initial stop after traceme, ignore it
    assert_eq!(
        root_pid,
        initial_pid,
        "First SIGTRAP was raised from another process than the root PID"
    );
    ptrace_mod::syscall(initial_pid).unwrap(); // wait for another

    loop {
        // detect enter, get syscall no
        let pid = wait_sigtrap!();
        if pid != root_pid {
            continue; // for the time being
        }

        let entry = map.entry(pid).or_insert(None);
        match entry.take() {
            None => {
                let no = ptrace_mod::peekuser(pid, ptrace_mod::Register::ORIG_RAX).unwrap();
                if let Some(ovride) = reg.find(no) {
                    let data = OverrideData {
                        data: (ovride.atenter)(pid),
                        syscall_no: no,
                    };
                    *entry = Some(data);
                }
            }
            Some(data) => {
                let ret = ptrace_mod::peekuser(pid, ptrace_mod::Register::ORIG_RAX).unwrap();
                if ret < 0 {
                    println!(
                        "Syscall {} exited with an error, not touching it",
                        data.syscall_no
                    );
                } else {
                    let ovride = reg.find(data.syscall_no).unwrap(); // if this was added to the map, the
                    (ovride.atexit)(pid, &data.data);
                }
            }
        }

        ptrace_mod::syscall(pid).unwrap(); // wait for another
    }
}
