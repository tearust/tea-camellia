
#![cfg(feature = "runtime-benchmarks")]

use sp_std::prelude::*;
use super::*;
use sp_runtime::traits::Bounded;
use frame_system::RawOrigin as SystemOrigin;
use frame_benchmarking::{
	benchmarks_instance_pallet, account, whitelisted_caller, whitelist_account, impl_benchmark_test_suite
};
use frame_support::traits::Get;
use frame_support::{traits::EnsureOrigin, dispatch::UnfilteredDispatchable};

use crate::Pallet as Assets;


