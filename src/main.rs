extern crate randmockery;

use randmockery::{parse_args, intercept_syscalls, patch_getrandom, syscall_table};
use randmockery::syscall_override::OverrideRegistry;

use std::process::Command;

fn main() {
    let args = parse_args();
    println!("Executing binary: {}", args[0]);
    let mut command = Command::new(&args[0]);
    command.args(&args[1..]);

    let mut reg = OverrideRegistry::new();
    reg.add(syscall_table::getrandom, |pid| patch_getrandom(pid, || 0));

    let exitcode = intercept_syscalls(command, reg);
    std::process::exit(exitcode as i32);
} 