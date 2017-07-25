extern crate randmockery;
extern crate nix;
extern crate rand;
extern crate libc;


#[cfg(test)]
mod tests {
    use randmockery::{intercept_syscalls, ptrace_setmem, spawn_child};
    use randmockery::syscall_override::OverrideRegistry;
    use randmockery::syscall_override::{getrandom, time};

    use std::process::Command;

    fn test_getrandom<F>(command: &str, expected_exitcode: i8, mut gen: F)
    where
        F: 'static + FnMut() -> u8,
    {
        let mut reg = OverrideRegistry::new();
        reg.add(
            ::libc::SYS_getrandom,
            getrandom::atenter,
            move |pid, data| ptrace_setmem(pid, data, &mut gen),
        );

        let pid = spawn_child(Command::new(command));
        let exitcode = intercept_syscalls(pid, reg);
        assert_eq!(exitcode, expected_exitcode);
    }

    #[test]
    fn constant_gen() {
        test_getrandom("tests/getrandom-test", 0, || 0);
        test_getrandom("tests/getrandom-test", 1, || 8);
    }

    #[test]
    fn random_gen() {
        use rand::{StdRng, SeedableRng, Rng};
        let mut rng = StdRng::from_seed(&[1, 2, 3, 4]);
        let gen = move || rng.gen::<u8>();

        test_getrandom("tests/getrandom-test-mocked", 0, gen);
    }

    #[test]
    fn test_logical_time() {
        let mut reg = OverrideRegistry::new();
        reg.add(::libc::SYS_time, time::atenter, time::atexit);

        let pid = spawn_child(Command::new("tests/time-test"));
        let exitcode = intercept_syscalls(pid, reg);
        assert_eq!(exitcode, 0);
    }

    #[test]
    fn test_logical_time_vdso() {
        let mut reg = OverrideRegistry::new();
        reg.add(::libc::SYS_time, time::atenter, time::atexit);

        let mut cmd = Command::new("tests/time-test-vdso");
        cmd.env("LD_PRELOAD", "tests/libmocktime.so");
        let pid = spawn_child(cmd);
        let exitcode = intercept_syscalls(pid, reg);
        assert_eq!(exitcode, 0);
    }
}
