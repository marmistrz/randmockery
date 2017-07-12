extern crate nix;
extern crate rand;

use std::process::Command;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;

mod ptrace_mod;
pub mod syscall_override;
pub mod syscall_table;

use syscall_override::OverrideRegistry;

/// if the process has finished: return its exit code
fn wait_sigtrap_fun(pid: Pid) -> Option<i8> {
    match waitpid(pid, None) {
        // TODO use PTRACE_O_TRACESYSGOOD
        // See this pull request: https://github.com/nix-rust/nix/pull/566
        Ok(WaitStatus::Exited(_, code)) => {
            println!("Inferior quit with code {}!", code);
            Some(code)
        }
        Ok(WaitStatus::Stopped(_, nix::sys::signal::Signal::SIGTRAP)) => None,
        Ok(s) => panic!("Unexpected stop reason: {:?}", s),
        Err(e) => panic!("Unexpected waitpid error: {:?}", e),
    }
}

fn ptrace_setmem<F>(pid: Pid, gen: &mut F, ptr: usize, len: usize)
where
    F: FnMut() -> u8,
{
    use std::mem;

    let step = mem::size_of::<usize>();

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

pub fn patch_getrandom<F>(pid: Pid, gen: &mut F)
where
    F: FnMut() -> u8,
{
    let bufptr = ptrace_mod::peekuser(pid, ptrace_mod::Register::RDI).unwrap() as usize;
    let buflen = ptrace_mod::peekuser(pid, ptrace_mod::Register::RSI).unwrap() as usize;
    println!("The inferior requested {} random bytes", buflen);

    ptrace_setmem(pid, gen, bufptr as usize, buflen);
}

fn spawn_child(mut command: Command) -> Pid {
    use ptrace_mod::PtraceSpawnable;

    let child = command.spawn_ptrace().expect(
        "Error spawning the child process",
    );

    Pid::from_raw(child.id() as i32) // This is awful, see https://github.com/nix-rust/nix/issues/656
}

macro_rules! wait_sigtrap {
    ($pid:ident) => (match wait_sigtrap_fun($pid) {
        None => {},
        Some(x) => return x
    })
}

/// Return value: exitcode
pub fn intercept_syscalls(command: Command, mut reg: OverrideRegistry) -> i8 {
    let pid = spawn_child(command);

    wait_sigtrap!(pid); // there will be an initial stop after traceme, ignore it
    ptrace_mod::syscall(pid).unwrap(); // wait for another

    loop {
        // detect enter, get syscall no
        wait_sigtrap!(pid);
        let no = ptrace_mod::peekuser(pid, ptrace_mod::Register::ORIG_RAX).unwrap();

        ptrace_mod::syscall(pid).unwrap(); // wait for another

        // detect exit, get exit code
        wait_sigtrap!(pid);
        let ret = ptrace_mod::peekuser(pid, ptrace_mod::Register::ORIG_RAX).unwrap();

        for ov in reg.iter_mut() {
            if no == ov.syscall {
                if ret < 0 {
                    println!("Syscall {} exited with an error, not touching it", no);
                } else {
                    (ov.atexit)(pid);
                }
            }
        }

        ptrace_mod::syscall(pid).unwrap(); // wait for another
    }
}