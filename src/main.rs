extern crate nix;
extern crate spawn_ptrace;

use std::process::Command;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;

use spawn_ptrace::CommandPtraceSpawn;

mod ptrace_mod;

fn main() {
    let child = Command::new("ls").arg("-l").spawn_ptrace().unwrap();
    let pid = Pid::from_raw(child.id() as i32);

    loop {
        let orig_rax = ptrace_mod::peekuser(pid, ptrace_mod::Register::RAX).unwrap();
        println!("We've got syscall: {}", orig_rax);

        ptrace_mod::syscall(pid).unwrap();

        match waitpid(pid, None) {
            Ok(WaitStatus::Exited(_, code)) => {
                println!("Inferior quit with code {}!", code);
                break;
            }
            Ok(WaitStatus::Stopped(_, nix::sys::signal::Signal::SIGTRAP)) => {}
            Ok(s) => panic!("Unexpected stop reason: {:?}", s),
            Err(e) => panic!("Unexpected waitpid error: {:?}", e),
        }
    }
}