extern crate nix;
extern crate spawn_ptrace;

use std::process::{Command, exit};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;

mod ptrace_mod;
mod syscall_table;

/// Return value: should we continue
fn do_wait(pid: Pid) {
    match waitpid(pid, None) {
        // TODO use PTRACE_O_TRACESYSGOOD
        // See this pull request: https://github.com/nix-rust/nix/pull/566
        Ok(WaitStatus::Exited(_, code)) => {
            println!("Inferior quit with code {}!", code);
            exit(code as i32)
        }
        Ok(WaitStatus::Stopped(_, nix::sys::signal::Signal::SIGTRAP)) => {}
        Ok(s) => panic!("Unexpected stop reason: {:?}", s),
        Err(e) => panic!("Unexpected waitpid error: {:?}", e),
    }
}

// TODO the name could've been better
/// Returns the syscall no
fn detect_syscall(pid: Pid) -> i64 {
    do_wait(pid);
    ptrace_mod::peekuser(pid, ptrace_mod::Register::ORIG_RAX).unwrap()
}

fn main() {
    use ptrace_mod::PtraceSpawnable;

    let child = Command::new("./getrandom").spawn_ptrace().expect(
        "Error spawning the child process",
    );
    let pid = Pid::from_raw(child.id() as i32); // This is awful, see https://github.com/nix-rust/nix/issues/656

    loop {
        let no = detect_syscall(pid); // detect enter, return syscall no
        ptrace_mod::syscall(pid).unwrap(); // wait for another

        let ret = detect_syscall(pid); // detect exit, return exit code
        if no == syscall_table::getrandom && ret >= 0 {
            println!("got getrandom!!");
            let bufptr = ptrace_mod::peekuser(pid, ptrace_mod::Register::RDI).unwrap();
            let buflen = ptrace_mod::peekuser(pid, ptrace_mod::Register::RSI).unwrap();

            println!("Inferior requested {} ramdom bytes", bufptr);
        }

        ptrace_mod::syscall(pid).unwrap(); // wait for another

    }
}