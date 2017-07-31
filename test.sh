#!/bin/sh
# Unfortunately, there's no way to specify the number of threads in Cargo.toml
RUST_TEST_THREADS=1 cargo test
