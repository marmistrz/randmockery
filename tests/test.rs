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
use std::sync::{mpsc, Mutex};
use std::thread;

use nix::{Error, Errno};
use nix::sys::signal::kill;

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

// This will grab the necessary mutex.
fn test_syscall<'a, F, G, S>(
    command: &str,
    expected_exitcode: i8,
    syscall_no: i64,
    atenter: F,
    atexit: G,
    preload: S,
) where
    F: 'static + FnMut(Pid) -> HandlerData,
    G: 'static + FnMut(Pid, &HandlerData) -> (),
    S: Into<Option<&'a str>>,
{
    get_mutex!();

    let mut reg = OverrideRegistry::new();
    reg.add(syscall_no, atenter, atexit);

    let mut cmd = Command::new(command);
    if let Some(pr) = preload.into() {
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
        "tests/libmocktime.so",
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

fn proc_lives(pid: Pid) -> nix::Result<bool> {
    match kill(pid, None) {
        Ok(_) => Ok(true),
        Err(Error::Sys(Errno::ESRCH)) => Ok(false),
        Err(e) => Err(e),
    }
}

#[test]
fn test_parent_death_kills_child() {
    get_mutex!();

    // a rendezvous channel
    let (tx, rx) = mpsc::sync_channel(0);

    let handle = thread::spawn(move || {
        use std::panic;
        // Don't be overly talkative
        panic::set_hook(Box::new(|_| println!("The supervisor panicked!")));

        let tid = nix::unistd::gettid();
        let child = spawn_child(Command::new("tests/crash-test"));
        tx.send(tid).unwrap();
        tx.send(child).unwrap();
        // the third send will be ignored, it's here just for synchronization
        tx.send(child).unwrap();
        intercept_syscalls(child, OverrideRegistry::new());
    });

    let supervisor_tid = rx.recv().unwrap();
    let inferior_pid = rx.recv().unwrap();

    // the supervisor is waiting, the inferior should be too
    assert_eq!(
        proc_lives(supervisor_tid),
        Ok(true),
        "Supervisor died before intercepting"
    );
    assert_eq!(
        proc_lives(inferior_pid),
        Ok(true),
        "Inferior died before intercepting"
    );

    // release the supervisor
    let _ = rx.recv().unwrap();

    // wait for the supervisor to crash
    match handle.join() {
        Err(_) => {}
        Ok(_) => panic!("The supervisor should have crashed"),
    }
    // wait for the inferior - otherwise it will show as defunct
    // and receive signals
    nix::sys::wait::waitpid(inferior_pid, None).unwrap();

    assert_eq!(
        proc_lives(supervisor_tid),
        Ok(false),
        "Supervisor did not crash"
    );
    assert_eq!(
        proc_lives(inferior_pid),
        Ok(false),
        "Inferior is still running"
    );
}
