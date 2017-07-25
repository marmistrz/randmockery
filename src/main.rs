extern crate randmockery;
extern crate libc;
extern crate rand;
extern crate libloading;

use randmockery::{intercept_syscalls, spawn_child, args};
use randmockery::syscall_override::OverrideRegistry;
use randmockery::syscall_override::{getrandom, time};

use std::process::Command;

trait InjectLib {
    fn inject_lib(&mut self, &str);
}

impl InjectLib for Command {
    fn inject_lib(&mut self, lib: &str) {
        use std::os::unix::process::CommandExt;
        let lib_s = String::from(lib);
        self.before_exec(move || {
            println!("Loading library {}", lib_s);
            match libloading::Library::new(&lib_s) {
                Err(err) => panic!("Error loading library: {}", err),
                Ok(_) => Ok(()),
            }
        });
    }
}

fn main() {
    let matches = args::get_parser().get_matches();

    let mut cmd = matches.values_of("command").unwrap();
    let prog = cmd.next().unwrap();
    let args: Vec<&str> = cmd.collect();

    println!("Executing binary: {}", prog);
    let mut command = Command::new(prog);
    command.args(&args);

    if let Some(preloads) = matches.values_of("library") {
        let ld_preload = preloads.collect::<Vec<_>>().join(":");
        println!("Setting LD_PRELOAD={}", ld_preload);
        command.env("LD_PRELOAD", ld_preload);
    }

    let mut reg = OverrideRegistry::new();

    reg.add(libc::SYS_getrandom, getrandom::atenter, getrandom::atexit);
    reg.add(libc::SYS_time, time::atenter, time::atexit);

    let pid = spawn_child(command);
    let exitcode = intercept_syscalls(pid, reg);

    std::process::exit(exitcode as i32);
}
