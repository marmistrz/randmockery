extern crate nix;

use nix::unistd::Pid;

pub mod getrandom;

type SyscallNo = i64;
pub struct HandlerData {
    pub bufptr: usize,
    pub buflen: usize,
}

pub struct SyscallOverride {
    /// syscall the override will match
    pub syscall: SyscallNo,
    pub atenter: Box<FnMut(Pid) -> HandlerData>,
    pub atexit: Box<FnMut(Pid, HandlerData) -> ()>,
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
        F: 'static + FnMut(Pid) -> HandlerData,
        G: 'static + FnMut(Pid, HandlerData) -> (),
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
            HandlerData {
                buflen: 0,
                bufptr: 0,
            }
        };
        let atexit = |_, _| {};

        reg.add(17, atenter, atexit);
        let el = reg.find(17).unwrap();
        assert_eq!(el.syscall, 17);
        let len = (el.atenter)(Pid::from_raw(17)).buflen;
        assert_eq!(len, 0);
    }
}
