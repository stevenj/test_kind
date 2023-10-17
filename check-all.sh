#!/bin/bash
# Simple script which tries to emulate the github action

# Set environment variable
export CARGO_TERM_COLOR=always

cd crate

echo Check code formatting - And Fix it.
cargo fmt
echo

echo Run Clippy lint checks
cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::cargo
echo

echo Build the project
cargo build --verbose
cargo build -r --verbose
echo

echo Build the docs
cargo doc -r --no-deps
echo

echo Run doc tests
cargo test --doc
echo

echo Run tests using cargo-nextest
cargo nextest run
echo