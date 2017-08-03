extern crate randmockery;
extern crate nix;
extern crate rand;
extern crate libc;
#[macro_use]
extern crate lazy_static;

use randmockery::{intercept_syscalls, ptrace_setmem, spawn_child};
use randmockery::syscall_override::{OverrideRegistry, HandlerData};
use randmockery::syscall_override::{getrandom, time};

use std::process::Command;
use std::sync::Mutex;

use nix::unistd::Pid;

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

fn test_syscall<F, G>(
    command: &str,
    expected_exitcode: i8,
    syscall_no: i64,
    atenter: F,
    atexit: G,
    preload: Option<&str>,
) where
    F: 'static + FnMut(Pid) -> HandlerData,
    G: 'static + FnMut(Pid, &HandlerData) -> (),
{
    get_mutex!();

    let mut reg = OverrideRegistry::new();
    reg.add(syscall_no, atenter, atexit);

    let mut cmd = Command::new(command);
    if let Some(pr) = preload {
        cmd.env("LD_PRELOAD", pr);
    };
    let pid = spawn_child(cmd);

    let exitcode = intercept_syscalls(pid, reg);
    assert_eq!(exitcode, expected_exitcode);
}

fn test_getrandom<F>(command: &str, expected_exitcode: i8, mut gen: F)
where
    F: 'static + FnMut() -> u8,
{
    test_syscall(
        command,
        expected_exitcode,
        ::libc::SYS_getrandom,
        getrandom::atenter,
        move |pid, data| ptrace_setmem(pid, data, &mut gen),
        None,
    )
}

#[test]
fn constant_gen_ok() {
    test_getrandom("tests/getrandom-test", 0, || 0);
}

#[test]
fn constant_gen_fail() {
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
fn intercept_forked_children() {
    use rand::{StdRng, SeedableRng, Rng};
    let mut rng = StdRng::from_seed(&[1, 2, 3, 4]);
    let gen = move || rng.gen::<u8>();

    test_getrandom("tests/getrandom-fork-test", 0, gen);
}

#[test]
fn intercept_threads() {
    test_getrandom("tests/getrandom-thread-test", 0, || 0);
}

#[test]
fn test_logical_time() {
    test_syscall(
        "tests/time-test",
        0,
        ::libc::SYS_time,
        time::time_atenter,
        time::time_atexit,
        None,
    );
}

#[test]
fn test_logical_time_vdso() {
    test_syscall(
        "tests/time-test-vdso",
        0,
        ::libc::SYS_time,
        time::time_atenter,
        time::time_atexit,
        Some("tests/time-test-vdso"),
    );
}

#[test]
fn test_clock_gettime() {
    test_syscall(
        "tests/clock_gettime-test",
        0,
        ::libc::SYS_clock_gettime,
        time::clock_gettime_atenter,
        time::clock_gettime_atexit,
        None,
    );
}

#[test]
fn test_gettimeofday() {
    test_syscall(
        "tests/gettimeofday-test",
        0,
        ::libc::SYS_gettimeofday,
        time::gettimeofday_atenter,
        time::gettimeofday_atexit,
        None,
    );
}
