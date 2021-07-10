// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use crate::index::{DOLLARS, STAKING_PRICE_TABLE};
use node_primitives::{AccountId, Balance};
use pallet_cml::{ServiceTaskPoint, StakingEconomics, StakingSnapshotItem};
use sp_std::cmp::min;
use sp_std::prelude::*;

pub mod index;
#[cfg(test)]
mod test;

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

	fn miner_total_staking_price(snapshots: &Vec<StakingSnapshotItem<AccountId>>) -> Balance {
		let total_slot_height = snapshots
			.last()
			.map(|item| item.staking_at + item.weight)
			.unwrap_or(1);

		let max_index = safe_index(total_slot_height);
		STAKING_PRICE_TABLE.iter().take(max_index).sum()
	}

	fn single_staking_reward(
		miner_total_rewards: Balance,
		total_staking_point: Balance,
		snapshot_item: &StakingSnapshotItem<AccountId>,
	) -> Balance {
		let mut staking_point = 0;
		for i in snapshot_item.staking_at..(snapshot_item.staking_at + snapshot_item.weight) {
			staking_point += STAKING_PRICE_TABLE[safe_index(i)];
		}
		miner_total_rewards * staking_point / total_staking_point
	}
}

fn safe_index(index: u32) -> usize {
	min(index as usize, STAKING_PRICE_TABLE.len())
}
