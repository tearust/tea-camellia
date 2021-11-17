use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub fn user_cml_list(who: &T::AccountId) -> Vec<u64> {
		UserCmlStore::<T>::iter_prefix(who)
			.map(|(id, _)| id)
			.collect()
	}

	pub fn user_staking_list(who: &T::AccountId) -> Vec<(u64, u64)> {
		let mut result = Vec::new();
		for (_, miner_item) in MinerItemStore::<T>::iter() {
			if !CmlStore::<T>::contains_key(miner_item.cml_id) {
				continue;
			}

			let cml = CmlStore::<T>::get(miner_item.cml_id);
			let cml_id = cml.id();
			let mut staking_list: Vec<(u64, u64)> = cml
				.staking_slots()
				.iter()
				.enumerate()
				.filter(|(_, staking_item)| staking_item.owner == *who)
				.map(|(index, _)| (cml_id, index as u64))
				.collect();
			result.append(&mut staking_list);
		}
		result
	}

	/// Returned item fields:
	/// - CML Id
	/// - Orbit Id
	/// - Cml type, utf-8 encoded string
	/// - Miner status, utf-8 encoded string
	/// - Machine Id
	pub fn current_mining_cml_list() -> Vec<(u64, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> {
		MinerItemStore::<T>::iter()
			.map(|(_, miner_item)| {
				let cml = CmlStore::<T>::get(miner_item.cml_id);
				let cml_type = match cml.cml_type() {
					CmlType::A => "A",
					CmlType::B => "B",
					CmlType::C => "C",
				};
				let machine_id = match cml.machine_id() {
					Some(machine_id) => machine_id.to_vec(),
					None => vec![],
				};
				let miner_status = match miner_item.status {
					MinerStatus::Active => "Active",
					MinerStatus::Offline => "Offline",
					MinerStatus::ScheduleDown => "ScheduleDown",
				};

				(
					miner_item.cml_id,
					miner_item.orbitdb_id.unwrap_or(vec![]),
					cml_type.as_bytes().to_vec(),
					miner_status.as_bytes().to_vec(),
					machine_id,
				)
			})
			.collect()
	}

	pub fn estimate_stop_mining_penalty(cml_id: u64) -> BalanceOf<T> {
		let cml = CmlStore::<T>::get(cml_id);
		let owner = cml.owner();

		let mut result: BalanceOf<T> = Default::default();
		for staking_item in cml.staking_slots() {
			if owner.is_some() && staking_item.owner.eq(owner.unwrap()) {
				continue;
			}

			let punishment = if let Some(cml_id) = staking_item.cml {
				T::StopMiningPunishment::get() * CmlStore::<T>::get(cml_id).staking_weight().into()
			} else {
				T::StopMiningPunishment::get()
			};

			result = result.saturating_add(punishment);
		}
		result
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::tests::new_genesis_seed;
	use crate::{
		CmlStore, MinerItem, MinerItemStore, MinerStatus, StakingCategory, StakingItem,
		StakingProperties, UserCmlStore, CML,
	};

	#[test]
	fn user_cml_list_works() {
		new_test_ext().execute_with(|| {
			let account1 = 1;
			let account2 = 2;

			let account1_cml1 = 11;
			let account1_cml2 = 12;
			let account1_cml3 = 13;

			let account2_cml1 = 21;
			let account2_cml2 = 22;
			let account2_cml3 = 23;
			let account2_cml4 = 24;

			UserCmlStore::<Test>::insert(account1, account1_cml1, ());
			UserCmlStore::<Test>::insert(account1, account1_cml2, ());
			UserCmlStore::<Test>::insert(account1, account1_cml3, ());

			UserCmlStore::<Test>::insert(account2, account2_cml1, ());
			UserCmlStore::<Test>::insert(account2, account2_cml2, ());
			UserCmlStore::<Test>::insert(account2, account2_cml3, ());
			UserCmlStore::<Test>::insert(account2, account2_cml4, ());

			let user1_cml_ids = Cml::user_cml_list(&1);
			assert_eq!(user1_cml_ids.len(), 3);
			assert!(user1_cml_ids.contains(&account1_cml1));
			assert!(user1_cml_ids.contains(&account1_cml2));
			assert!(user1_cml_ids.contains(&account1_cml3));

			let user2_cml_ids = Cml::user_cml_list(&2);
			assert_eq!(user2_cml_ids.len(), 4);
			assert!(user2_cml_ids.contains(&account2_cml1));
			assert!(user2_cml_ids.contains(&account2_cml2));
			assert!(user2_cml_ids.contains(&account2_cml3));
			assert!(user2_cml_ids.contains(&account2_cml4));
		})
	}

	#[test]
	fn user_staking_list_works() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let user3 = 3;

			let cml_id1 = 1;
			let cml_id2 = 2;
			let cml_id3 = 3;
			let cml_id4 = 4;
			let cml_id5 = 5;

			let mut cml1 = CML::from_genesis_seed(new_genesis_seed(cml_id1));
			cml1.staking_slots_mut().push(StakingItem {
				owner: user1,
				category: StakingCategory::Tea,
				amount: Some(1),
				cml: None,
			});
			cml1.staking_slots_mut().push(StakingItem {
				owner: user3,
				category: StakingCategory::Tea,
				amount: Some(1),
				cml: None,
			});
			CmlStore::<Test>::insert(cml_id1, cml1);

			let mut cml2 = CML::from_genesis_seed(new_genesis_seed(cml_id2));
			cml2.staking_slots_mut().push(StakingItem {
				owner: user2,
				category: StakingCategory::Tea,
				amount: Some(1),
				cml: None,
			});
			cml2.staking_slots_mut().push(StakingItem {
				owner: user2,
				category: StakingCategory::Cml,
				amount: None,
				cml: Some(cml_id3),
			});
			cml2.staking_slots_mut().push(StakingItem {
				owner: user1,
				category: StakingCategory::Cml,
				amount: None,
				cml: Some(cml_id4),
			});
			cml2.staking_slots_mut().push(StakingItem {
				owner: user1,
				category: StakingCategory::Cml,
				amount: None,
				cml: Some(cml_id5),
			});
			CmlStore::<Test>::insert(cml_id2, cml2);

			CmlStore::<Test>::insert(cml_id3, CML::from_genesis_seed(new_genesis_seed(cml_id3)));
			CmlStore::<Test>::insert(cml_id4, CML::from_genesis_seed(new_genesis_seed(cml_id4)));
			CmlStore::<Test>::insert(cml_id5, CML::from_genesis_seed(new_genesis_seed(cml_id5)));

			let machine_id1 = [1; 32];
			let machine_id2 = [2; 32];

			MinerItemStore::<Test>::insert(
				machine_id1,
				MinerItem {
					cml_id: cml_id1,
					id: machine_id1,
					ip: vec![],
					controller_account: Default::default(),
					orbitdb_id: None,
					status: MinerStatus::Active,
					suspend_height: None,
					schedule_down_height: None,
				},
			);
			MinerItemStore::<Test>::insert(
				machine_id2,
				MinerItem {
					cml_id: cml_id2,
					id: machine_id2,
					ip: vec![],
					controller_account: Default::default(),
					orbitdb_id: None,
					status: MinerStatus::Active,
					suspend_height: None,
					schedule_down_height: None,
				},
			);

			let user1_staking_list = Cml::user_staking_list(&user1);
			assert_eq!(user1_staking_list.len(), 3);
			assert!(user1_staking_list.contains(&(cml_id1, 0)));
			assert!(user1_staking_list.contains(&(cml_id2, 2)));
			assert!(user1_staking_list.contains(&(cml_id2, 3)));

			let user2_staking_list = Cml::user_staking_list(&user2);
			assert_eq!(user2_staking_list.len(), 2);
			assert!(user2_staking_list.contains(&(cml_id2, 0)));
			assert!(user2_staking_list.contains(&(cml_id2, 1)));

			let user3_staking_list = Cml::user_staking_list(&user3);
			assert_eq!(user3_staking_list.len(), 1);
			assert_eq!(user3_staking_list[0], (cml_id1, 1));
		})
	}
}
