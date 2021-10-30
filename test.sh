#!/usr/bin/env sh

cargo test -p pallet-tea --lib
cargo test -p pallet-cml --lib
cargo test -p pallet-auction --lib
cargo test -p pallet-genesis-bank --lib
cargo test -p pallet-genesis-exchange --lib
cargo test -p pallet-bonding-curve --lib
cargo test -p pallet-utils --lib
cargo test -p bonding-curve-impl --lib
cargo test -p pallet-staking --lib