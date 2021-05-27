// Copyright 2019-2021 Tea Project
//! Autogenerated weights for pallet_tea
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-05-27, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/tea-camellia
// benchmark
// --chain=dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_tea
// --extrinsic=*
// --steps=50
// --repeat=20
// --heap-pages=4096
// --header=./file_header.txt
// --output=runtime/src/weights/pallet_tea.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_tea.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_tea::WeightInfo for WeightInfo<T> {
    fn add_new_node() -> Weight {
        (25_718_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}
