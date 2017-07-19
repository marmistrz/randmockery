extern crate randmockery;
extern crate libc;
extern crate rand;

use randmockery::{parse_args, intercept_syscalls};
use randmockery::syscall_override::OverrideRegistry;
use randmockery::syscall_override::{getrandom, time};

use std::process::Command;

fn main() {
    let args = parse_args();
    println!("Executing binary: {}", args[0]);
    let mut command = Command::new(&args[0]);
    command.args(&args[1..]);

    let mut reg = OverrideRegistry::new();

    reg.add(libc::SYS_getrandom, getrandom::atenter, getrandom::atexit);
    reg.add(libc::SYS_time, time::atenter, time::atexit);

    let exitcode = intercept_syscalls(command, reg);

    std::process::exit(exitcode as i32);
}
