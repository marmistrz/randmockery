extern crate nix;

use nix::unistd::Pid;
use nix::{libc, Result};

pub mod getrandom;
pub mod time;

pub type SyscallNo = i64;

/// An enum containing all needed variants of data which needs to be stored
/// by a handler.
/// For best extensibility, we'd have used runtime polymorphism with trait
/// objects, but this was a PoC.
pub enum HandlerData {
    Buffer { bufptr: usize, buflen: usize },
    Timespec(*mut libc::timespec),
    None,
}

/// This struct fully describes the actions that need to be done
/// when a syscall with number `syscall` gets intercepted.
/// The routines `atenter` and `atexit` are executed on relevant events.
///
/// If a ptrace error occurs, the functions should not panic but just
/// propagate the error.
pub struct SyscallOverride {
    /// syscall the override will match
    pub syscall: SyscallNo,
    pub atenter: Box<FnMut(Pid) -> Result<HandlerData>>,
    pub atexit: Box<FnMut(Pid, &HandlerData) -> Result<()>>,
}

pub struct OverrideRegistry {
    pub overrides: Vec<SyscallOverride>,
}

impl OverrideRegistry {
    pub fn new() -> Self {
        OverrideRegistry { overrides: Vec::new() }
    }

    pub fn add<F, G>(&mut self, syscall: SyscallNo, atenter: F, atexit: G) -> &mut Self
    where
        F: 'static + FnMut(Pid) -> Result<HandlerData>,
        G: 'static + FnMut(Pid, &HandlerData) -> Result<()>,
    {
        self.overrides.push(SyscallOverride {
            syscall: syscall,
            atenter: Box::new(atenter),
            atexit: Box::new(atexit),
        });
        self
    }

    pub fn find(&mut self, no: SyscallNo) -> Option<&mut SyscallOverride> {
        self.overrides.iter_mut().find(|ov| ov.syscall == no)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let mut reg = OverrideRegistry::new();
        let atenter = |_| {
            Ok(HandlerData::Buffer {
                buflen: 0,
                bufptr: 0,
            })
        };
        let atexit = |_, _: &_| Ok(());

        reg.add(17, atenter, atexit);
        let el = reg.find(17).unwrap();
        assert_eq!(el.syscall, 17);
        let len = match (el.atenter)(Pid::from_raw(17)).unwrap() {
            HandlerData::Buffer { buflen, .. } => buflen,
            _ => panic!(),
        };
        assert_eq!(len, 0);
    }
}
