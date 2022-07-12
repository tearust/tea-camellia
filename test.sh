#!/usr/bin/env sh

set -e

cargo test -p pallet-machine --lib
cargo test -p pallet-cml --lib
cargo test -p pallet-tea-erc20 --lib
cargo test -p pallet-utils --lib

