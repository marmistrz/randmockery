extern crate randmockery;

use randmockery::{parse_args, intercept_syscalls, getrandom};
use randmockery::syscall_override::OverrideRegistry;

extern crate rand;

use std::process::Command;

fn main() {
    let args = parse_args();
    println!("Executing binary: {}", args[0]);
    let mut command = Command::new(&args[0]);
    command.args(&args[1..]);

    let mut reg = OverrideRegistry::new();

    reg.add(getrandom::SYSCALL_NO, getrandom::atenter, getrandom::atexit);

    let exitcode = intercept_syscalls(command, reg);

    std::process::exit(exitcode as i32);
}
