extern crate randmockery;

#[cfg(test)]
mod tests {
    use randmockery::{intercept_syscalls, patch_getrandom, syscall_table};
    use randmockery::syscall_override::OverrideRegistry;

    #[test]
    fn integration() {
        use std::process::Command;

        let status = Command::new("make")
            .args(&["-C", "tests", "getrandom-test"])
            .status()
            .unwrap();
        assert!(status.success());


        {
            let mut reg = OverrideRegistry::new();
            reg.add(syscall_table::getrandom, |pid| patch_getrandom(pid, || 0));

            let exitcode = intercept_syscalls(Command::new("tests/getrandom-test"), reg);
            assert_eq!(exitcode, 0);
        }


        {
            let mut reg = OverrideRegistry::new();
            reg.add(syscall_table::getrandom, |pid| patch_getrandom(pid, || 8));

            let exitcode = intercept_syscalls(Command::new("tests/getrandom-test"), reg);
            assert!(exitcode != 0);
        }
    }
}