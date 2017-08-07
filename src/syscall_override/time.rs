//! override for sys_time

//! Module for patching the getrandom syscall
//!
//! This takes care of both getrandom(2) and getentropy(2) system calls since both of them use
//! the sys_getrandom syscall.
extern crate nix;

use nix::Result;
use nix::unistd::Pid;
use syscall_override::HandlerData;
use ptrace_mod;
use nix::libc;

fn logical_time() -> i64 {
    use std::cell::RefCell;

    thread_local! {
        static LOGICAL_TIME: RefCell<i64> = RefCell::new(0);
    }

    LOGICAL_TIME.with(|cell| {
        let mut mref = cell.borrow_mut();
        let time = *mref;
        *mref += 1;
        time
    })
}

pub fn time_atenter(_: Pid) -> Result<HandlerData> {
    Ok(HandlerData::None {})
}

pub fn time_atexit(pid: Pid, _: &HandlerData) -> Result<()> {
    ptrace_mod::pokeuser(pid, ptrace_mod::Register::RAX, logical_time() as u64)
}

pub fn time_atexit_allzero(pid: Pid, _: &HandlerData) -> Result<()> {
    ptrace_mod::pokeuser(pid, ptrace_mod::Register::RAX, 0u64)
}

pub fn clock_gettime_atenter(pid: Pid) -> Result<HandlerData> {
    let ptr = ptrace_mod::peekuser(pid, ptrace_mod::Register::RSI)? as *mut libc::timespec;
    Ok(HandlerData::Timespec(ptr))
}

pub fn clock_gettime_atexit(pid: Pid, data: &HandlerData) -> Result<()> {
    use std::mem;
    use ptrace_setmem;

    let_extract!(
        HandlerData::Timespec(ptr),
        *data,
        panic!("Mismatched HandlerData variant")
    );
    let buffer = HandlerData::Buffer {
        bufptr: ptr as usize,
        buflen: mem::size_of::<libc::timespec>(),
    };

    ptrace_setmem(pid, &buffer, &mut || 0)
}

pub fn gettimeofday_atenter(pid: Pid) -> Result<HandlerData> {
    let ptr = ptrace_mod::peekuser(pid, ptrace_mod::Register::RDI)? as *mut libc::timespec;
    print!("{:?}", ptr);
    Ok(HandlerData::Timespec(ptr)) // timeval has a compatible signature
}

pub fn gettimeofday_atexit(pid: Pid, data: &HandlerData) -> Result<()> {
    use std::mem;
    use ptrace_setmem;

    let_extract!(
        HandlerData::Timespec(ptr),
        *data,
        panic!("Mismatched HandlerData variant")
    );
    let buffer = HandlerData::Buffer {
        bufptr: ptr as usize,
        buflen: mem::size_of::<libc::timeval>(),
    };

    ptrace_setmem(pid, &buffer, &mut || 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_time() {
        for i in 0..5 {
            assert_eq!(logical_time(), i);
        }
    }
}
