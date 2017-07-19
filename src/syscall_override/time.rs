//! override for sys_time

//! Module for patching the getrandom syscall
//!
//! This takes care of both getrandom(2) and getentropy(2) system calls since both of them use
//! the sys_getrandom syscall.
extern crate nix;
extern crate libc;

use nix::unistd::Pid;
use syscall_override::HandlerData;
use ptrace_mod;

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

pub fn atenter(_: Pid) -> HandlerData {
    HandlerData::None {}
}

pub fn atexit(pid: Pid, _: HandlerData) {
    ptrace_mod::pokeuser(pid, ptrace_mod::Register::RAX, logical_time() as u64).unwrap()
}

// TODO use the libc constant when it gets merged
// see:
pub const SYSCALL_NO: i32 = libc::SYS_time;

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
