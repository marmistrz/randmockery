extern crate randmockery;
extern crate nix;
extern crate rand;
extern crate libc;
#[macro_use]
extern crate lazy_static;

use randmockery::{intercept_syscalls, ptrace_setmem, spawn_child};
use randmockery::syscall_override::OverrideRegistry;
use randmockery::syscall_override::{getrandom, time};

use std::process::Command;
use std::sync::Mutex;

lazy_static! {
    /// Any test that creates child processes must grab this mutex, regardless
    /// of what it does with those children.
    pub static ref MTX: Mutex<()> = Mutex::new(());
}

macro_rules! get_mutex {
    () => {
        let _guard = MTX.lock();
    }
}

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
    get_mutex!();

    test_getrandom("tests/getrandom-test", 0, || 0);
    test_getrandom("tests/getrandom-test", 1, || 8);
}

#[test]
fn random_gen() {
    get_mutex!();

    use rand::{StdRng, SeedableRng, Rng};
    let mut rng = StdRng::from_seed(&[1, 2, 3, 4]);
    let gen = move || rng.gen::<u8>();

    test_getrandom("tests/getrandom-test-mocked", 0, gen);
}

#[test]
fn intercept_forked_children() {
    get_mutex!();

    use rand::{StdRng, SeedableRng, Rng};
    let mut rng = StdRng::from_seed(&[1, 2, 3, 4]);
    let gen = move || rng.gen::<u8>();

    test_getrandom("tests/getrandom-fork-test", 0, gen);
}

#[test]
fn intercept_threads() {
    get_mutex!();

    test_getrandom("tests/getrandom-thread-test", 0, || 0);
}

#[test]
fn test_logical_time() {
    get_mutex!();

    let mut reg = OverrideRegistry::new();
    reg.add(::libc::SYS_time, time::time_atenter, time::time_atexit);

    let pid = spawn_child(Command::new("tests/time-test"));
    let exitcode = intercept_syscalls(pid, reg);
    assert_eq!(exitcode, 0);
}

#[test]
fn test_logical_time_vdso() {
    get_mutex!();

    let mut reg = OverrideRegistry::new();
    reg.add(::libc::SYS_time, time::time_atenter, time::time_atexit);

    let mut cmd = Command::new("tests/time-test-vdso");
    cmd.env("LD_PRELOAD", "tests/libmocktime.so");
    let pid = spawn_child(cmd);
    let exitcode = intercept_syscalls(pid, reg);
    assert_eq!(exitcode, 0);
}

#[test]
fn test_clock_gettime() {
    get_mutex!();

    let mut reg = OverrideRegistry::new();
    reg.add(
        ::libc::SYS_clock_gettime,
        time::clock_gettime_atenter,
        time::clock_gettime_atexit,
    );

    let pid = spawn_child(Command::new("tests/clock_gettime-test"));
    let exitcode = intercept_syscalls(pid, reg);
    assert_eq!(exitcode, 0);
}

#[test]
fn test_gettimeofday() {
    get_mutex!();

    let mut reg = OverrideRegistry::new();
    reg.add(
        ::libc::SYS_gettimeofday,
        time::gettimeofday_atenter,
        time::gettimeofday_atexit,
    );

    let pid = spawn_child(Command::new("tests/gettimeofday-test"));
    let exitcode = intercept_syscalls(pid, reg);
    assert_eq!(exitcode, 0);
}
