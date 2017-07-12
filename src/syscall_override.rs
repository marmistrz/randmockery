extern crate nix;

use nix::unistd::Pid;
use std::slice::IterMut;

pub struct SyscallOverride {
    /// syscall the override will match
    pub syscall: i64,
    pub atexit: Box<FnMut(Pid) -> ()>,
}

pub struct OverrideRegistry {
    pub overrides: Vec<SyscallOverride>,
}

impl OverrideRegistry {
    pub fn new() -> Self {
        OverrideRegistry { overrides: Vec::new() }
    }

    pub fn add<F>(&mut self, syscall: i64, atexit: F) -> &mut Self
    where
        F: 'static + FnMut(Pid) -> (),
    {
        self.overrides.push(SyscallOverride {
            syscall,
            atexit: Box::new(atexit),
        });
        self
    }

    pub fn iter_mut(&mut self) -> IterMut<SyscallOverride> {
        self.overrides.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let mut reg = OverrideRegistry::new();
        reg.add(17, |_| {});
        let el = reg.iter_mut().next().unwrap();
        assert_eq!(el.syscall, 17);
    }
}