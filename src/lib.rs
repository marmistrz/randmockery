extern crate nix;
extern crate rand;
#[macro_use]
extern crate enum_extract;
extern crate libloading;
#[macro_use]
extern crate clap;
extern crate prctl;

use std::process::Command;
use std::collections::HashMap;

use nix::{Error, Errno, Result};
use nix::unistd::Pid;
use nix::sys::ptrace;
use nix::sys::signal::Signal;
use nix::sys::wait::{wait, WaitStatus};

mod ptrace_mod;
pub mod syscall_override;
pub mod args;

use syscall_override::{OverrideRegistry, HandlerData, SyscallNo};

trait KillChildrenOnDeath {
    fn kill_children_on_death(&mut self) -> &mut Self;
}

impl KillChildrenOnDeath for Command {
    fn kill_children_on_death(&mut self) -> &mut Self {
        use std::os::unix::process::CommandExt;
        self.before_exec(|| {
            prctl::set_death_signal(Signal::SIGHUP as isize).unwrap();
            Ok(())
        })
    }
}

pub fn spawn_child(mut command: Command) -> Pid {
    use ptrace_mod::Ptraceable;

    let child = command.kill_children_on_death().ptrace().spawn().expect(
        "Error spawning the child process",
    );

    Pid::from_raw(child.id() as i32) // This is awful, see https://github.com/nix-rust/nix/issues/656
}

pub struct OverrideData {
    pub syscall_no: SyscallNo,
    pub data: HandlerData,
}
type ProcInfo = HashMap<Pid, Option<OverrideData>>;

fn handle_syscall(
    status: WaitStatus,
    map: &mut ProcInfo,
    reg: &mut OverrideRegistry,
) -> Result<Option<i8>> {
    let pid = match status {
        WaitStatus::Exited(pid, code) => {
            println!("Process {} quit with code {}!", pid, code);
            map.remove(&pid);
            return Ok(Some(code));
        }
        WaitStatus::PtraceSyscall(pid) => {
            let entry = map.entry(pid).or_insert_with(
                || panic!("Unexpected pid: {}", pid),
            );

            let rax = ptrace_mod::peekuser(pid, ptrace_mod::Register::ORIG_RAX)?;

            match entry.take() {
                None => {
                    let no = rax;
                    if let Some(ovride) = reg.find(no) {
                        let data = OverrideData {
                            data: (ovride.atenter)(pid)?,
                            syscall_no: no,
                        };
                        *entry = Some(data);
                    }
                }
                Some(data) => {
                    let ret = rax;
                    if ret < 0 {
                        println!(
                            "Syscall {} exited with an error, not touching it",
                            data.syscall_no
                        );
                    } else {
                        // if there's an entry in the map, there must have been
                        // an override too
                        let ovride = reg.find(data.syscall_no).unwrap();
                        (ovride.atexit)(pid, &data.data)?;
                    }
                    *entry = None;
                }
            };
            pid
        }
        WaitStatus::PtraceEvent(pid, _, _) => {
            println!("{:?}", status);
            pid
        }
        WaitStatus::Stopped(pid, sig @ Signal::SIGSTOP) => {
            println!("{:?}", status);
            // FIXME process may receive SIGSTOP for another reason
            if !map.contains_key(&pid) {
                map.insert(pid, None);
            } else {
                println!("Inferior received a signal: {:?}", sig)
            }
            pid
        }
        WaitStatus::Stopped(pid, sig) => {
            println!("Inferior received a signal: {:?}", sig);
            pid
        }
        s => panic!("Unexpected stop reason: {:?}", s),
    };

    ptrace_mod::syscall(pid)?;
    Ok(None)
}

/// Return value: exitcode
pub fn intercept_syscalls(root_pid: Pid, mut reg: OverrideRegistry) -> i8 {
    let mut map: ProcInfo = HashMap::new();
    map.insert(root_pid, None);

    let flags = ptrace::ptrace::PTRACE_O_TRACESYSGOOD | ptrace::ptrace::PTRACE_O_TRACECLONE |
        ptrace::ptrace::PTRACE_O_TRACEFORK;

    assert_eq!(
        wait(),
        Ok(WaitStatus::Stopped(root_pid, Signal::SIGTRAP)),
        "Unexpected process caught by wait()"
    );
    // setoptions must be called on a stopped process!
    ptrace::setoptions(root_pid, flags).expect("Failed to set tracing options");
    ptrace_mod::syscall(root_pid).expect("Process died before tracing"); // wait for another

    let mut exitcode = None;
    while map.len() > 0 {
        // detect enter, get syscall no
        let status = wait().expect("wait() failed");
        match handle_syscall(status, &mut map, &mut reg) {
            Ok(code @ Some(_)) => {
                if status.pid().unwrap() == root_pid {
                    assert_eq!(exitcode, None, "Exitcode was set twice");
                    exitcode = code;
                }
            }
            Ok(None) => {}
            Err(Error::Sys(Errno::ESRCH)) => {
                println!(
                    "Warning: process {} died while a syscall was being processed.",
                    status.pid().unwrap()
                )
            }
            Err(e) => panic!("handle_syscall: unexpected error: {}", e),
        }
    }
    exitcode.expect("Child process did not exit for some reason")
}

pub fn ptrace_setmem<F>(pid: Pid, data: &HandlerData, gen: &mut F) -> Result<()>
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
            ptrace_mod::pokedata(pid, curr, genword())?;
            curr += step;
            next += step;
        }
    }


    let lastword = ptrace_mod::peekdata(pid, curr)?;
    let numzero = end - curr;
    let newword: u64;

    unsafe {
        let mut bytes: [u8; 8] = mem::transmute(lastword);
        for i in 0..numzero {
            bytes[i] = gen();
        }
        newword = mem::transmute(bytes);
    }

    ptrace_mod::pokedata(pid, curr, newword)?;
    Ok(())
}
