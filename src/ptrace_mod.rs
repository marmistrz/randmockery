//! This is here only temporarily
//! All of this should finally be merged into nix, it's where it belongs

//! Only `PTRACE_PEEK*` return an actual result, so only these will return `nix::Result<c_long>`.
//! All the others will just return `Result<()>`.

extern crate nix;

use std;

use nix::libc::{c_long, c_void};
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::*;
use nix::unistd::Pid;

use std::ptr;
use std::process::{Child, Command};

use std::os::unix::process::CommandExt;

#[cfg(target_arch = "x86_64")]
// We're going to export it anyway
#[allow(dead_code)]
pub enum Register {
    R15 = 0 * 8,
    R14 = 1 * 8,
    R13 = 2 * 8,
    R12 = 3 * 8,
    RBP = 4 * 8,
    RBX = 5 * 8,
    R11 = 6 * 8,
    R10 = 7 * 8,
    R9 = 8 * 8,
    R8 = 9 * 8,
    RAX = 10 * 8,
    RCX = 11 * 8,
    RDX = 12 * 8,
    RSI = 13 * 8,
    RDI = 14 * 8,
    ORIG_RAX = 15 * 8,
    RIP = 16 * 8,
    CS = 17 * 8,
    EFLAGS = 18 * 8,
    RSP = 19 * 8,
    SS = 20 * 8,
    FS_BASE = 21 * 8,
    GS_BASE = 22 * 8,
    DS = 23 * 8,
    ES = 24 * 8,
    FS = 25 * 8,
    GS = 26 * 8,
}

/// Makes the `PTRACE_SYSCALL` request to ptrace
pub fn syscall(pid: Pid) -> nix::Result<()> {
    ptrace(PTRACE_SYSCALL, pid, ptr::null_mut(), ptr::null_mut()).map(|_| ()) // ignore the useless return value
}

/// Makes the `PTRACE_PEEKUSER` request to ptrace
pub fn peekuser(pid: Pid, reg: Register) -> nix::Result<c_long> {
    let reg_arg = (reg as i32) as *mut c_void;
    ptrace(PTRACE_PEEKUSER, pid, reg_arg, ptr::null_mut())
}

/// Convenience function to peek n-th syscall argument
/// The arguments are indexed from 0
/// Only arguments passed via registers are supported
pub fn peek_nth_syscall_arg(pid: Pid, no: u64) -> nix::Result<c_long> {
    fn arg(no: u64) -> self::Register {
        use self::Register::*;
        match no {
            0 => RDI,
            1 => RSI,
            2 => R10,
            3 => R8,
            4 => R9,
            _ => panic!(""),
        }
    }
    peekuser(pid, arg(no))
}



/// Sets the process as traceable with `PTRACE_TRACEME`
pub fn traceme() -> nix::Result<()> {
    ptrace(
        PTRACE_TRACEME,
        Pid::from_raw(0),
        ptr::null_mut(),
        ptr::null_mut(),
    ).map(|_| ()) // ignore the useless return value
}

pub trait PtraceSpawnable {
    fn spawn_ptrace(&mut self) -> std::io::Result<Child>;
}

impl PtraceSpawnable for Command {
    fn spawn_ptrace(&mut self) -> std::io::Result<Child> {
        self.before_exec(|| {
            traceme().expect("Error initalizing ptrace in the child process");
            Ok(())
        }).spawn()
    }
}