extern crate nix;

use nix::unistd::Pid;
use std::slice::Iter;

pub struct SyscallOverride {
    /// syscall the override will match
    pub syscall: i64,
    pub atexit: Box<Fn(Pid) -> ()>,
}

pub struct OverrideRegistry {
    pub overrides: Vec<SyscallOverride>,
}

impl OverrideRegistry {
    pub fn new() -> Self {
        OverrideRegistry { overrides: Vec::new() }
    }

    pub fn add<F>(mut self, syscall: i64, atexit: F) -> Self
    where
        F: 'static + Fn(Pid) -> (),
    {
        self.overrides.push(SyscallOverride {
            syscall,
            atexit: Box::new(atexit),
        });
        self
    }

    pub fn iter(&self) -> Iter<SyscallOverride> {
        self.overrides.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let reg = OverrideRegistry::new().add(17, |pid| println!("it works"));
        let el = reg.iter().next();
    }
}