use crate::index::{DOLLARS, STAKING_PRICE_TABLE, STAKING_SLOTS_MAX_LENGTH};
use crate::{safe_index, TeaStakingEconomics};
use node_primitives::AccountId;
use pallet_cml::{StakingEconomics, StakingSnapshotItem};

#[test]
fn miner_total_staking_price_works() {
	let snapshots = vec![
		StakingSnapshotItem {
			owner: AccountId::default(),
			staking_at: 0,
			weight: 1,
		},
		StakingSnapshotItem {
			owner: AccountId::default(),
			staking_at: 1,
			weight: 3,
		},
		StakingSnapshotItem {
			owner: AccountId::default(),
			staking_at: 4,
			weight: 2,
		},
	];

	let mut total = 0;
	for i in 0..6 {
		total += STAKING_PRICE_TABLE[i];
	}

	assert_eq!(
		TeaStakingEconomics::miner_total_staking_price(&snapshots),
		total
	);
}

#[test]
fn miner_total_staking_price_works_if_snapshot_is_empty() {
	assert_eq!(TeaStakingEconomics::miner_total_staking_price(&vec![]), 0);
}

#[test]
fn single_staking_reward_works() {
	let total_balance = 10 * DOLLARS;
	let staking_count = 10;
	let mut total_staking_point = 0;
	for i in 0..staking_count {
		total_staking_point += STAKING_PRICE_TABLE[i];
	}

	let mut real_total_balance = 0;
	for i in 0..staking_count {
		let balance = TeaStakingEconomics::single_staking_reward(
			total_balance,
			total_staking_point,
			&StakingSnapshotItem {
				owner: AccountId::default(),
				weight: 1,
				staking_at: i as u32,
			},
		);
		assert_eq!(
			balance,
			total_balance * STAKING_PRICE_TABLE[i] / total_staking_point
		);

		real_total_balance += balance;
	}

	// due to integer division, real_total_balance is always small than planed issuance
	assert!(real_total_balance < total_balance);
	// however the gap is very small
	assert!(total_balance - real_total_balance < staking_count as u128);
}

#[test]
fn single_staking_reward_works_if_weight_larger_than_one() {
	let staking_count = 10;
	let mut total_staking_point = 0;
	for i in 0..staking_count {
		total_staking_point += STAKING_PRICE_TABLE[i];
	}

	assert_eq!(
		TeaStakingEconomics::single_staking_reward(
			total_staking_point,
			total_staking_point,
			&StakingSnapshotItem {
				owner: AccountId::default(),
				weight: 2,
				staking_at: 1,
			}
		),
		STAKING_PRICE_TABLE[1] + STAKING_PRICE_TABLE[2]
	);

	assert_eq!(
		TeaStakingEconomics::single_staking_reward(
			total_staking_point,
			total_staking_point,
			&StakingSnapshotItem {
				owner: AccountId::default(),
				weight: 3,
				staking_at: 5,
			}
		),
		STAKING_PRICE_TABLE[5] + STAKING_PRICE_TABLE[6] + STAKING_PRICE_TABLE[7]
	);
}

#[test]
fn single_staking_reward_works_if_has_zero_value() {
	assert_eq!(
		TeaStakingEconomics::single_staking_reward(
			0,
			10000,
			&StakingSnapshotItem {
				owner: AccountId::default(),
				weight: 3,
				staking_at: 5,
			}
		),
		0
	);

	assert_eq!(
		TeaStakingEconomics::single_staking_reward(
			10000,
			0,
			&StakingSnapshotItem {
				owner: AccountId::default(),
				weight: 3,
				staking_at: 5,
			}
		),
		0
	);

	assert_eq!(
		TeaStakingEconomics::single_staking_reward(
			0,
			0,
			&StakingSnapshotItem {
				owner: AccountId::default(),
				weight: 3,
				staking_at: 5,
			}
		),
		0
	);
}

#[test]
fn safe_index_works() {
	assert_eq!(safe_index(0), 0);
	assert_eq!(safe_index(1), 1);
	assert_eq!(safe_index(1023), 1023);
	assert_eq!(safe_index(1024), STAKING_SLOTS_MAX_LENGTH as usize);
	assert_eq!(safe_index(1025), STAKING_SLOTS_MAX_LENGTH as usize);
	assert_eq!(safe_index(u32::MAX), STAKING_SLOTS_MAX_LENGTH as usize);
}
