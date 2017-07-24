Build status: [![CircleCI](https://circleci.com/gh/marmistrz/randmockery.svg?style=svg)](https://circleci.com/gh/marmistrz/randmockery)

A proof of concept that we can mock randomness.

## How does it work?

We use `ptrace` to get notified about any syscall the inferior makes.
When the inferior enters or exits a syscall, a user-defined handler is executed.

This gives us the opportunity to replace the syscall return value/buffer with our
own data.

## Benchmarks
* `cmp file1 file2` on 1.5G files yielded 8x slowdown (this is the pessimistic scenario of an application, which does mostly syscalls)
* running a compute-only job (calculating `1/(1+x)` using a series for a 1000 values of x with double precision) showed virtually no slowndown

## git dependencies
Unfortunately, we depend on `git` versions of some crates:

* nix-rust/nix: we need `struct Pid`
* rust-lang/libc: we need the system call table

## upstreaming
`ptrace_mod.rs` will be upstreamed and is currently under review nix-rust/nix#666
