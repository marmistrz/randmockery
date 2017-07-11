extern crate randmockery;

use randmockery::{parse_args, intercept_syscalls, patch_getrandom, syscall_table};
use randmockery::syscall_override::OverrideRegistry;

fn main() {
    let command = parse_args();

    // TODO: modularize more. We'd like to test the loop with mocked OverrideRegistry
    let mut reg = OverrideRegistry::new();
    reg.add(syscall_table::getrandom, |pid| patch_getrandom(pid, || 0));

    let exitcode = intercept_syscalls(command, reg);
    std::process::exit(exitcode as i32);
}