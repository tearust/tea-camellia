// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use crate::index::DOLLARS;
use node_primitives::{AccountId, Balance};
use pallet_cml::{MinerStakingPoint, ServiceTaskPoint, StakingEconomics, StakingSnapshotItem};
use sp_std::prelude::*;

pub mod index;

pub struct TeaStakingEconomics {}

impl Default for TeaStakingEconomics {
	fn default() -> Self {
		TeaStakingEconomics {}
	}
}

impl StakingEconomics<Balance, AccountId> for TeaStakingEconomics {
	fn increase_issuance(total_point: ServiceTaskPoint) -> Balance {
		(total_point as Balance) * DOLLARS
	}

	fn total_staking_rewards_of_miner(
		miner_point: ServiceTaskPoint,
		_total_point: ServiceTaskPoint,
	) -> Balance {
		(miner_point as Balance) * DOLLARS
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
