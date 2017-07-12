extern crate randmockery;
extern crate nix;
extern crate rand;


#[cfg(test)]
mod tests {
    use randmockery::{intercept_syscalls, patch_getrandom, syscall_table};
    use randmockery::syscall_override::OverrideRegistry;

    use std::process::Command;

    fn test_instance<F>(command: &str, expected_exitcode: i8, mut gen: F)
    where
        F: 'static + FnMut() -> u8,
    {
        let mut reg = OverrideRegistry::new();
        reg.add(syscall_table::getrandom, move |pid| {
            patch_getrandom(pid, &mut gen)
        });

        let exitcode = intercept_syscalls(Command::new(command), reg);
        assert_eq!(exitcode, expected_exitcode);
    }

    #[test]
    fn constant_gen() {

        let status = Command::new("make")
            .args(&["-C", "tests", "getrandom-test"])
            .status()
            .unwrap();
        assert!(status.success());


        test_instance("tests/getrandom-test", 0, || 0);
        test_instance("tests/getrandom-test", 1, || 8);


        {
            let mut reg = OverrideRegistry::new();
            reg.add(syscall_table::getrandom, |pid| {
                patch_getrandom(pid, &mut || 8)
            });

            let exitcode = intercept_syscalls(Command::new("tests/getrandom-test"), reg);
            assert!(exitcode != 0);
        }
    }

    #[test]
    fn random_gen() {
        let status = Command::new("make")
            .args(&["-C", "tests", "getrandom-test-mocked"])
            .status()
            .unwrap();
        assert!(status.success());

        use rand::{StdRng, SeedableRng, Rng};
        let mut rng = StdRng::from_seed(&[1, 2, 3, 4]);
        let gen = move || rng.gen::<u8>();

        test_instance("tests/getrandom-test-mocked", 0, gen);
    }
}