#!/usr/bin/env sh

set -e

cargo test -p pallet-tea --lib
cargo test -p pallet-cml --lib
cargo test -p pallet-bonding-curve --lib
cargo test -p pallet-utils --lib
cargo test -p pallet-staking --lib

