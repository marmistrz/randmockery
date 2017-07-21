Build status: [![CircleCI](https://circleci.com/gh/marmistrz/randmockery.svg?style=svg)](https://circleci.com/gh/marmistrz/randmockery)

A proof of concept that we can mock randomness.

## git dependencies
Unfortunately, we depend on `git` versions of some crates:

* nix-rust/nix: we need `struct Pid`
* rust-lang/libc: we need the system call table

## upstreaming
`ptrace_mod.rs` will be upstreamed and is currently under review nix-rust/nix#666
