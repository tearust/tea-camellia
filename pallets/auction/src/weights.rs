// #![cfg_attr(rustfmt, rustfmt_skip)]
// #![allow(unused_parens)]
// #![allow(unused_imports)]
// #![allow(clippy::unnecessary_cast)]

// use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
// use sp_std::marker::PhantomData;

// /// Weight functions needed for orml_auction.
// pub trait WeightInfo {
// 	fn bid_collateral_auction() -> Weight;
// 	fn on_finalize(c: u32, ) -> Weight;
// }

// /// Default weights.
// impl WeightInfo for () {
// 	fn bid_collateral_auction() -> Weight {
// 		(108_000_000 as Weight)
// 			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
// 			.saturating_add(RocksDbWeight::get().writes(9 as Weight))
// 	}
// 	fn on_finalize(c: u32, ) -> Weight {
// 		(9_779_000 as Weight)
// 			// Standard Error: 13_000
// 			.saturating_add((57_962_000 as Weight).saturating_mul(c as Weight))
// 			.saturating_add(RocksDbWeight::get().reads(10 as Weight))
// 			.saturating_add(RocksDbWeight::get().reads((3 as Weight).saturating_mul(c as Weight)))
// 			.saturating_add(RocksDbWeight::get().writes(7 as Weight))
// 			.saturating_add(RocksDbWeight::get().writes((3 as Weight).saturating_mul(c as Weight)))
// 	}
// }
