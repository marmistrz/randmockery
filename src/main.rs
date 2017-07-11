extern crate nix;

use std::process::{Command, exit};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;

mod ptrace_mod;
mod syscall_override;
mod syscall_table;

use syscall_override::OverrideRegistry;

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

fn ptrace_setmem<F>(pid: Pid, gen: F, ptr: usize, len: usize)
where
    F: Fn() -> u8,
{
    use std::mem;

    let genword = || -> u64 {
        let mut word: [u8; 8] = [0; 8];
        for x in word.iter_mut() {
            *x = gen();
        }

        unsafe { mem::transmute(word) }
    };

    let step = mem::size_of::<usize>();

    let end = ptr + len;
    let mut curr = ptr;
    let mut next = curr + step;

    while next < end {
        ptrace_mod::pokedata(pid, curr, genword()).unwrap();
        curr += step;
        next += step;
    }

    let lastword = ptrace_mod::peekdata(pid, curr).unwrap();
    let numzero = end - curr;
    let newword: u64;

    unsafe {
        let mut bytes: [u8; 8] = mem::transmute(lastword);
        for i in 0..numzero {
            bytes[i] = gen();
        }
        newword = mem::transmute(bytes);
    }

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

fn patch_getrandom<F>(pid: Pid, gen: F)
where
    F: Fn() -> u8,
{
    let bufptr = ptrace_mod::peekuser(pid, ptrace_mod::Register::RDI).unwrap() as usize;
    let buflen = ptrace_mod::peekuser(pid, ptrace_mod::Register::RSI).unwrap() as usize;
    println!("The inferior requested {} random bytes", buflen);

    ptrace_setmem(pid, gen, bufptr as usize, buflen);
}

fn spawn_child(command: Vec<String>) -> Pid {
    use ptrace_mod::PtraceSpawnable;

    let child = Command::new(&command[0])
        .args(&command[1..])
        .spawn_ptrace()
        .expect("Error spawning the child process");

    Pid::from_raw(child.id() as i32) // This is awful, see https://github.com/nix-rust/nix/issues/656
}

fn intercept_syscalls(command: Vec<String>, reg: OverrideRegistry) {
    println!("Executing binary: {}", command[0]);
    let pid = spawn_child(command);

    wait_sigtrap(pid); // there will be an initial stop after traceme, ignore it
    ptrace_mod::syscall(pid).unwrap(); // wait for another

    loop {
        let no = detect_syscall(pid); // detect enter, return syscall no
        ptrace_mod::syscall(pid).unwrap(); // wait for another

        let ret = detect_syscall(pid); // detect exit, return exit code

        for ov in reg.iter() {
            if no == ov.syscall {
                if ret < 0 {
                    println!("getrandom exited with an error, not touching it");
                } else {
                    (ov.atexit)(pid);
                }
            }
        }

        ptrace_mod::syscall(pid).unwrap(); // wait for another
    }
}

fn main() {
    let command = parse_args();

    // TODO: modularize more. We'd like to test the loop with mocked OverrideRegistry
    let mut reg = OverrideRegistry::new();
    reg.add(syscall_table::getrandom, |pid| patch_getrandom(pid, || 0));

    intercept_syscalls(command, reg);
}