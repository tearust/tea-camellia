// Copyright (C) 2021 Tea Project.

//! Autogenerated weights for pallet_cml
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-07-17, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/tea-camellia
// benchmark
// --chain=dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_cml
// --extrinsic=*
// --steps=50
// --repeat=20
// --heap-pages=4096
// --header=./file_header.txt
// --output=pallets/cml/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cml.
pub trait WeightInfo {
	fn transfer_coupon() -> Weight;
	fn draw_investor_cmls_from_coupon() -> Weight;
	fn draw_team_cmls_from_coupon() -> Weight;
	fn active_cml() -> Weight;
	fn start_mining() -> Weight;
	fn stop_mining() -> Weight;
	fn start_balance_staking() -> Weight;
	fn start_cml_staking() -> Weight;
	fn stop_balance_staking(s: u32, ) -> Weight;
	fn stop_cml_staking(s: u32, ) -> Weight;
	fn withdraw_staking_reward() -> Weight;
	fn pay_off_mining_credit() -> Weight;
	fn dummy_ra_task() -> Weight;
}

/// Weights for pallet_cml using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn transfer_coupon() -> Weight {
		(26_049_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn draw_investor_cmls_from_coupon() -> Weight {
		(783_609_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(47 as Weight))
			.saturating_add(T::DbWeight::get().writes(86 as Weight))
	}
	fn draw_team_cmls_from_coupon() -> Weight {
		(1_127_650_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(67 as Weight))
			.saturating_add(T::DbWeight::get().writes(126 as Weight))
	}
	fn active_cml() -> Weight {
		(38_420_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn start_mining() -> Weight {
		(44_783_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn stop_mining() -> Weight {
		(29_234_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn start_balance_staking() -> Weight {
		(63_136_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn start_cml_staking() -> Weight {
		(55_194_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn stop_balance_staking(_s: u32, ) -> Weight {
		(690_487_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn stop_cml_staking(_s: u32, ) -> Weight {
		(11_257_113_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1027 as Weight))
			.saturating_add(T::DbWeight::get().writes(1026 as Weight))
	}
	fn withdraw_staking_reward() -> Weight {
		(22_493_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn pay_off_mining_credit() -> Weight {
		(48_847_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn dummy_ra_task() -> Weight {
		(24_511_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn transfer_coupon() -> Weight {
		(26_049_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn draw_investor_cmls_from_coupon() -> Weight {
		(783_609_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(47 as Weight))
			.saturating_add(RocksDbWeight::get().writes(86 as Weight))
	}
	fn draw_team_cmls_from_coupon() -> Weight {
		(1_127_650_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(67 as Weight))
			.saturating_add(RocksDbWeight::get().writes(126 as Weight))
	}
	fn active_cml() -> Weight {
		(38_420_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn start_mining() -> Weight {
		(44_783_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes(3 as Weight))
	}
	fn stop_mining() -> Weight {
		(29_234_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn start_balance_staking() -> Weight {
		(63_136_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn start_cml_staking() -> Weight {
		(55_194_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn stop_balance_staking(_s: u32, ) -> Weight {
		(690_487_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn stop_cml_staking(_s: u32, ) -> Weight {
		(11_257_113_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1027 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1026 as Weight))
	}
	fn withdraw_staking_reward() -> Weight {
		(22_493_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn pay_off_mining_credit() -> Weight {
		(48_847_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn dummy_ra_task() -> Weight {
		(24_511_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
}