// Copyright 2019-2021 Tea Project
//! Autogenerated weights for pallet_election_provider_multi_phase
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
// --pallet=pallet_election_provider_multi_phase
// --extrinsic=*
// --steps=50
// --repeat=20
// --heap-pages=4096
// --header=./file_header.txt
// --output=runtime/src/weights/pallet_election_provider_multi_phase.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_election_provider_multi_phase.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_election_provider_multi_phase::WeightInfo for WeightInfo<T> {
	fn on_initialize_nothing() -> Weight {
		(21_820_000 as Weight).saturating_add(T::DbWeight::get().reads(7 as Weight))
	}
	fn on_initialize_open_signed() -> Weight {
		(108_827_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn on_initialize_open_unsigned_with_snapshot() -> Weight {
		(108_343_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn on_initialize_open_unsigned_without_snapshot() -> Weight {
		(19_863_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn elect_queued() -> Weight {
		(7_937_790_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn submit_unsigned(v: u32, _t: u32, a: u32, d: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 21_000
			.saturating_add((4_073_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 21_000
			.saturating_add((12_268_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 106_000
			.saturating_add((3_394_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn feasibility_check(v: u32, t: u32, a: u32, d: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 10_000
			.saturating_add((4_212_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 35_000
			.saturating_add((485_000 as Weight).saturating_mul(t as Weight))
			// Standard Error: 10_000
			.saturating_add((9_366_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 52_000
			.saturating_add((3_811_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
	}
}
