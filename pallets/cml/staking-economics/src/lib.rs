// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use crate::index::{DOLLARS, STAKING_PRICE_TABLE};
use node_primitives::{AccountId, Balance};
use pallet_cml::{Performance, ServiceTaskPoint, StakingEconomics, StakingSnapshotItem};
use sp_runtime::traits::Zero;
use sp_std::cmp::min;
use sp_std::prelude::*;

pub mod index;
#[cfg(test)]
mod test;

const TASK_POINT_BASE: Balance = 1000;
const PERFORMANCE_BASE: Balance = 10000;

pub struct TeaStakingEconomics {}

impl Default for TeaStakingEconomics {
	fn default() -> Self {
		TeaStakingEconomics {}
	}
}

impl StakingEconomics<Balance, AccountId> for TeaStakingEconomics {
	/// Calculate issuance balance with given total task point of current staking window.
	fn increase_issuance(total_point: ServiceTaskPoint) -> Balance {
		(total_point as Balance) * DOLLARS
	}

	/// Calculate total staking rewards of the given miner, the staking rewards should split to all staking
	/// users.
	fn total_staking_rewards_of_miner(
		miner_point: ServiceTaskPoint,
		_total_point: ServiceTaskPoint,
		performance: Performance,
	) -> Balance {
		(miner_point as Balance) * DOLLARS * (performance as Balance)
			/ TASK_POINT_BASE
			/ PERFORMANCE_BASE
	}

	/// Calculate all staking weight about the given miner.
	fn miner_total_staking_weight(snapshots: &Vec<StakingSnapshotItem<AccountId>>) -> Balance {
		let total_slot_height = snapshots
			.last()
			.map(|item| item.staking_at + item.weight)
			.unwrap_or(0);

		let max_index = safe_index(total_slot_height);
		if max_index == 0 {
			return Zero::zero();
		}

		STAKING_PRICE_TABLE.iter().take(max_index).sum()
	}

	/// Calculate a single staking reward.
	fn single_staking_reward(
		miner_total_rewards: Balance,
		total_staking_point: Balance,
		snapshot_item: &StakingSnapshotItem<AccountId>,
	) -> Balance {
		if miner_total_rewards.is_zero() || total_staking_point.is_zero() {
			return Zero::zero();
		}

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
