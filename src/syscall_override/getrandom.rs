//! Module for patching the getrandom syscall
//!
//! This takes care of both getrandom(2) and getentropy(2) system calls since both of them use
//! the sys_getrandom syscall.
extern crate nix;
extern crate rand;
extern crate libc;

use nix::unistd::Pid;
use nix::Result;

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

pub fn atenter(pid: Pid) -> Result<HandlerData> {
    Ok(HandlerData::Buffer {
        bufptr: ptrace_mod::peekuser(pid, ptrace_mod::Register::RDI)? as usize,
        buflen: ptrace_mod::peekuser(pid, ptrace_mod::Register::RSI)? as usize,
    })
}

pub fn atexit(pid: Pid, data: &HandlerData) -> Result<()> {
    ptrace_setmem(pid, data, &mut random_byte)
}

pub fn atexit_allzero(pid: Pid, data: &HandlerData) -> Result<()> {
    ptrace_setmem(pid, data, &mut || 0)
}
