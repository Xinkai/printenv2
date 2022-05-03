#!/bin/sh

set -eu

cargo clippy --all-targets --all-features -- --deny warnings --deny clippy::all --deny clippy::pedantic --deny clippy::nursery --deny clippy::cargo
cargo fmt --check
