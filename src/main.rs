extern crate nix;

use std::process::{Command, exit};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;

mod ptrace_mod;
mod syscall_table;

/// Return value: should we continue
fn wait_sigtrap(pid: Pid) {
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
    wait_sigtrap(pid);
    ptrace_mod::peekuser(pid, ptrace_mod::Register::ORIG_RAX).unwrap()
}

fn ptrace_zero_mem(pid: Pid, ptr: usize, len: usize) {
    use std::mem;
    let step = mem::size_of::<usize>();

    let end = ptr + len;
    let mut curr = ptr;
    let mut next = curr + step;

    while next < end {
        ptrace_mod::pokedata(pid, curr, 0).unwrap();
        curr += step;
        next += step;
    }

    let lastword = ptrace_mod::peekdata(pid, curr).unwrap();
    let mut bytes: [u8; 8] = unsafe { mem::transmute(lastword) };
    let numzero = end - curr;
    for i in 0..numzero {
        bytes[i] = 0;
    }
    let newword: u64 = unsafe { mem::transmute(bytes) };

    ptrace_mod::pokedata(pid, curr, newword).unwrap();
}

fn parse_args() -> Vec<String> {
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

fn main() {
    use ptrace_mod::PtraceSpawnable;

    let command = parse_args();

    println!("Executing binary: {}", command[0]);
    let child = Command::new(&command[0])
        .args(&command[1..])
        .spawn_ptrace()
        .expect("Error spawning the child process");
    let pid = Pid::from_raw(child.id() as i32); // This is awful, see https://github.com/nix-rust/nix/issues/656

    wait_sigtrap(pid); // there will be an initial stop after traceme, ignore it
    ptrace_mod::syscall(pid).unwrap(); // wait for another

    loop {
        let no = detect_syscall(pid); // detect enter, return syscall no
        ptrace_mod::syscall(pid).unwrap(); // wait for another

        let ret = detect_syscall(pid); // detect exit, return exit code
        if no == syscall_table::getrandom {
            if ret < 0 {
                println!("getrandom exited with an error, not touching it");
            } else {
                println!("got getrandom!!");
                let bufptr = ptrace_mod::peekuser(pid, ptrace_mod::Register::RDI).unwrap() as usize;
                let buflen = ptrace_mod::peekuser(pid, ptrace_mod::Register::RSI).unwrap() as usize;

                let num = ptrace_mod::peekdata(pid, bufptr).unwrap() as u64;
                println!("The inferior received the number: {}", num);

                ptrace_zero_mem(pid, bufptr as usize, buflen);
            }
        }

        ptrace_mod::syscall(pid).unwrap(); // wait for another

    }
}