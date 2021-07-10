// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use node_primitives::{AccountId, Balance};
use pallet_cml::{MinerStakingPoint, ServiceTaskPoint, StakingEconomics, StakingSnapshotItem};
use sp_runtime::traits::Zero;
use sp_std::prelude::*;

pub struct TeaStakingEconomics {}

impl Default for TeaStakingEconomics {
	fn default() -> Self {
		TeaStakingEconomics {}
	}
}

impl StakingEconomics<Balance, AccountId> for TeaStakingEconomics {
	fn increase_issuance(_total_point: ServiceTaskPoint) -> Balance {
		// todo implement me later
		Zero::zero()
	}

	fn total_staking_rewards_of_miner(
		_miner_point: ServiceTaskPoint,
		_total_point: ServiceTaskPoint,
	) -> Balance {
		// todo implement me later
		Zero::zero()
	}

	fn miner_staking_points(
		snapshots: &Vec<StakingSnapshotItem<AccountId>>,
	) -> Vec<(AccountId, MinerStakingPoint)> {
		let total_slot_height = snapshots
			.last()
			.map(|item| item.staking_at + item.weight)
			.unwrap_or_default();

		snapshots
			.iter()
			.map(|item| {
				let base_point = total_slot_height - item.staking_at;
				(item.owner.clone(), base_point * base_point * item.weight)
			})
			.collect()
	}
}
