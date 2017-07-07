extern crate colored;
extern crate nix;
extern crate spawn_ptrace;

use std::process::Command;
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::*;
use nix::sys::wait::{wait, WaitStatus};
use std::ptr;
use nix::libc::c_void;

use spawn_ptrace::CommandPtraceSpawn;

const RAX: i64 = 8 * 15;

fn main() {
    let child = Command::new("ls").arg("-l").spawn_ptrace().unwrap();
    let pid = child.id() as i32;

    loop {
        let orig_rax = ptrace(PTRACE_PEEKUSER, pid, RAX as *mut c_void, ptr::null_mut()).unwrap();
        println!("We've got syscall: {}", orig_rax);

        ptrace(PTRACE_SYSCALL, pid, ptr::null_mut(), ptr::null_mut()).unwrap();

        match wait() {
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