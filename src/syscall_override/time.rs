//! override for sys_time

//! Module for patching the getrandom syscall
//!
//! This takes care of both getrandom(2) and getentropy(2) system calls since both of them use
//! the sys_getrandom syscall.
extern crate nix;

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

pub fn time_atenter(_: Pid) -> HandlerData {
    HandlerData::None {}
}

pub fn time_atexit(pid: Pid, _: HandlerData) {
    ptrace_mod::pokeuser(pid, ptrace_mod::Register::RAX, logical_time() as u64).unwrap()
}

pub fn clock_gettime_atenter(pid: Pid) -> HandlerData {
    let ptr = ptrace_mod::peekuser(pid, ptrace_mod::Register::RSI).unwrap() as *mut libc::timespec;
    HandlerData::Timespec(ptr)
}

pub fn clock_gettime_atexit(pid: Pid, data: HandlerData) {
    use std::mem;
    use ptrace_setmem;

    let_extract!(
        HandlerData::Timespec(ptr),
        data,
        panic!("Mismatched HandlerData variant")
    );
    let buffer = HandlerData::Buffer {
        bufptr: ptr as usize,
        buflen: mem::size_of::<libc::timespec>(),
    };

    ptrace_setmem(pid, buffer, &mut || 0)

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
