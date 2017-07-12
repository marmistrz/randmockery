extern crate randmockery;

use randmockery::{parse_args, intercept_syscalls, patch_getrandom, syscall_table};
use randmockery::syscall_override::OverrideRegistry;

extern crate rand;

use std::process::Command;
use rand::{Rng, SeedableRng, StdRng};

fn main() {
    let args = parse_args();
    println!("Executing binary: {}", args[0]);
    let mut command = Command::new(&args[0]);
    command.args(&args[1..]);

    let mut rng = StdRng::from_seed(&[1, 2, 3, 4]);
    let mut gen = move || rng.gen::<u8>();
    let atexit = move |pid| patch_getrandom(pid, &mut gen);

    let mut reg = OverrideRegistry::new();

    reg.add(syscall_table::getrandom, atexit);

    let exitcode = intercept_syscalls(command, reg);

    std::process::exit(exitcode as i32);
}