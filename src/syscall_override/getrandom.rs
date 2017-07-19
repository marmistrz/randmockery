//! Module for patching the getrandom syscall
//!
//! This takes care of both getrandom(2) and getentropy(2) system calls since both of them use
//! the sys_getrandom syscall.
extern crate nix;
extern crate rand;
extern crate libc;

use nix::unistd::Pid;

use ptrace_setmem;
use ptrace_mod;
use syscall_override::HandlerData;

fn random_byte() -> u8 {
    use std::cell::RefCell;
    use rand::{Rng, SeedableRng, StdRng};

    thread_local! {
        static RNG: RefCell<StdRng> = RefCell::new(StdRng::from_seed(&[1, 2, 3, 4]));
    }

    RNG.with(|cell| cell.borrow_mut().gen::<u8>())
}

pub fn atenter(pid: Pid) -> HandlerData {
    HandlerData::Buffer {
        bufptr: ptrace_mod::peekuser(pid, ptrace_mod::Register::RDI).unwrap() as usize,
        buflen: ptrace_mod::peekuser(pid, ptrace_mod::Register::RSI).unwrap() as usize,
    }
}

pub fn atexit(pid: Pid, data: HandlerData) {
    ptrace_setmem(pid, data, &mut random_byte);
}
