use super::*;

impl<T: tea::Config> tea::Pallet<T> {
	pub(crate) fn should_update_validators(n: &T::BlockNumber) -> bool {
		*n % T::UpdateValidatorsDuration::get() == 0u32.into()
	}

	pub(crate) fn update_runtime_status(block_number: T::BlockNumber) {
		for (tea_id, node) in Nodes::<T>::iter() {
			if node.status == NodeStatus::Active {
				if block_number - node.update_time <= T::RuntimeActivityThreshold::get().into() {
					continue;
				}

				Nodes::<T>::mutate(&tea_id, |node| node.status = NodeStatus::Inactive);
			}
		}
	}

	pub(crate) fn update_validators() {
		let active_machines: Vec<TeaPubKey> = T::CmlOperation::current_mining_cmls()
			.iter()
			.filter(|(_, tea_id)| Nodes::<T>::get(tea_id).is_active())
			.map(|(_, tea_id)| tea_id.clone())
			.collect();
		ValidatorsCollection::<T>::set(active_machines.clone());

		Self::deposit_event(Event::RaValidatorsChanged(active_machines));
	}

	pub(crate) fn is_validator(
		tea_id: &TeaPubKey,
		target_tea_id: &TeaPubKey,
		block_number: &T::BlockNumber,
	) -> bool {
		let groups_info = Self::generate_groups(block_number);

		// determine group id randomly by tea id hash
		let group_id = Self::group_id(target_tea_id, groups_info.len());
		match groups_info.get(&group_id) {
			Some(group) => group.contains(tea_id),
			None => false,
		}
	}

	pub(crate) fn generate_groups(
		block_number: &T::BlockNumber,
	) -> BTreeMap<u32, BTreeSet<TeaPubKey>> {
		if !frame_system::BlockHash::<T>::contains_key(block_number) {
			return Default::default();
		}

		let (validators_count, group_count, last_group_insufficient_member) =
			Self::parse_group_params();

		let block_hash = frame_system::BlockHash::<T>::get(block_number);
		let mut tea_id_hash_numbers = Vec::new();
		ValidatorsCollection::<T>::get().iter().for_each(|tea_id| {
			tea_id_hash_numbers.push((tea_id.clone(), Self::hash_number(&block_hash, tea_id)));
		});

		tea_id_hash_numbers.sort_by(|(_, a), (_, b)| a.cmp(b));
		let sorted_tea_ids: Vec<TeaPubKey> = tea_id_hash_numbers
			.into_iter()
			.map(|(tea_id, _)| tea_id)
			.collect();

		let mut general_groups: BTreeMap<u32, BTreeSet<TeaPubKey>> = BTreeMap::new();
		(0..group_count).into_iter().for_each(|id| {
			general_groups.insert(id, Default::default());
		});

		let mut has_substituted = false;
		let mut current_group_index = 0u32;
		for i in 0..validators_count {
			if Self::should_begin_substitution(
				i,
				validators_count,
				last_group_insufficient_member,
				&mut has_substituted,
			) || Self::should_normally_change_group(i, has_substituted)
			{
				current_group_index += 1;
			}

			if let Some(array) = general_groups.get_mut(&current_group_index) {
				array.insert(sorted_tea_ids[i as usize]);
			}
		}

		general_groups
	}

	pub(crate) fn update_validator_groups_count() {
		ValidatorGroupsCount::<T>::remove_all(None);

		let (validators_count, _group_count, last_group_insufficient_member) =
			Self::parse_group_params();

		let mut current_group_index = 0u32;
		let mut current_group_length = 0u32;
		let mut has_substituted = false;
		for i in 0..validators_count {
			if Self::should_begin_substitution(
				i,
				validators_count,
				last_group_insufficient_member,
				&mut has_substituted,
			) || Self::should_normally_change_group(i, has_substituted)
			{
				ValidatorGroupsCount::<T>::insert(current_group_index, current_group_length);

				current_group_length = 0;
				current_group_index += 1;
			}

			current_group_length += 1;
		}
		ValidatorGroupsCount::<T>::insert(current_group_index, current_group_length);
	}

	pub(crate) fn group_id(target_tea_id: &TeaPubKey, groups_count: usize) -> u32 {
		(Self::h256_to_u64(&H256::from_slice(&target_tea_id[..])) % groups_count as u64) as u32
	}

	pub(crate) fn should_normally_change_group(index: u32, has_substituted: bool) -> bool {
		!has_substituted && index != 0 && index % T::MaxGroupMemberCount::get() == 0
	}

	pub(crate) fn should_begin_substitution(
		index: u32,
		validators_count: u32,
		last_group_insufficient: bool,
		has_substituted: &mut bool,
	) -> bool {
		let result =
			last_group_insufficient && validators_count - index == T::MinGroupMemberCount::get();
		if result {
			*has_substituted = true;
		}
		result
	}

	pub(crate) fn parse_group_params() -> (u32, u32, bool) {
		let validators_count = ValidatorsCollection::<T>::get().len() as u32;
		let group_count = validators_count / T::MaxGroupMemberCount::get() + 1;
		let last_group_count = validators_count % T::MaxGroupMemberCount::get();
		let last_group_insufficient_number = last_group_count < T::MinGroupMemberCount::get();

		(
			validators_count,
			group_count,
			last_group_insufficient_number,
		)
	}

	pub(crate) fn hash_number(block_hash: &T::Hash, tea_id: &TeaPubKey) -> u64 {
		let payload = (block_hash, tea_id);
		let hash: H256 = payload.using_encoded(blake2_256).into();
		Self::h256_to_u64(&hash)
	}

	pub(crate) fn h256_to_u64(hash: &H256) -> u64 {
		const SIZE_OF_U64: usize = 8;
		const SIZE_OF_H256: usize = 32;
		let mut u8_buf = [0u8; SIZE_OF_U64];
		u8_buf.copy_from_slice(&hash.0[SIZE_OF_H256 - SIZE_OF_U64..SIZE_OF_H256]);
		u64::from_le_bytes(u8_buf)
	}
}

#[cfg(test)]
mod tests {
	use crate::{mock::*, types::*, ValidatorGroupsCount, ValidatorsCollection};
	use sp_core::H256;
	use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

	#[test]
	fn parse_group_params_works() {
		new_test_ext().execute_with(|| {
			ValidatorsCollection::<Test>::set(vec![[0u8; 32]; 7]);
			assert_eq!(Tea::parse_group_params(), (7, 1, false));

			ValidatorsCollection::<Test>::set(vec![[0u8; 32]; 11]);
			assert_eq!(Tea::parse_group_params(), (11, 2, true));

			ValidatorsCollection::<Test>::set(vec![[0u8; 32]; 15]);
			assert_eq!(Tea::parse_group_params(), (15, 2, false));

			ValidatorsCollection::<Test>::set(vec![[0u8; 32]; 21]);
			assert_eq!(Tea::parse_group_params(), (21, 3, true));
		})
	}

	#[test]
	fn should_normally_change_group_works() {
		new_test_ext().execute_with(|| {
			for i in 0..MAX_GROUP_MEMBER_COUNT {
				assert!(!Tea::should_normally_change_group(i, false));
			}
			assert!(Tea::should_normally_change_group(
				MAX_GROUP_MEMBER_COUNT,
				false
			));

			for i in MAX_GROUP_MEMBER_COUNT + 1..2 * MAX_GROUP_MEMBER_COUNT {
				assert!(!Tea::should_normally_change_group(i, false));
			}
			assert!(Tea::should_normally_change_group(
				2 * MAX_GROUP_MEMBER_COUNT,
				false
			));
		})
	}

	#[test]
	fn should_begin_substitution_works() {
		new_test_ext().execute_with(|| {
			let mut has_substituted = false;
			let validators_count = 11;
			for i in 0..6 {
				assert!(!Tea::should_begin_substitution(
					i,
					validators_count,
					true,
					&mut has_substituted
				));
			}
			assert!(!has_substituted);

			assert!(Tea::should_begin_substitution(
				6,
				validators_count,
				true,
				&mut has_substituted
			));
			assert!(has_substituted);

			has_substituted = false;
			for i in 7..11 {
				assert!(!Tea::should_begin_substitution(
					i,
					validators_count,
					true,
					&mut has_substituted
				));
			}
			assert!(!has_substituted);

			let validators_count = 15;
			for i in 0..validators_count {
				assert!(!Tea::should_begin_substitution(
					i,
					validators_count,
					false,
					&mut has_substituted
				));
			}
			assert!(!has_substituted);
		})
	}

	#[test]
	fn update_validator_groups_count_works() {
		new_test_ext().execute_with(|| {
			ValidatorsCollection::<Test>::set(vec![[0u8; 32]; 7]);
			Tea::update_validator_groups_count();
			assert_eq!(ValidatorGroupsCount::<Test>::get(0), 7);

			ValidatorsCollection::<Test>::set(vec![[0u8; 32]; 11]);
			Tea::update_validator_groups_count();
			assert_eq!(ValidatorGroupsCount::<Test>::get(0), 6);
			assert_eq!(ValidatorGroupsCount::<Test>::get(1), 5);

			ValidatorsCollection::<Test>::set(vec![[0u8; 32]; 15]);
			Tea::update_validator_groups_count();
			assert_eq!(ValidatorGroupsCount::<Test>::get(0), 10);
			assert_eq!(ValidatorGroupsCount::<Test>::get(1), 5);

			ValidatorsCollection::<Test>::set(vec![[0u8; 32]; 21]);
			Tea::update_validator_groups_count();
			assert_eq!(ValidatorGroupsCount::<Test>::get(0), 10);
			assert_eq!(ValidatorGroupsCount::<Test>::get(1), 6);
			assert_eq!(ValidatorGroupsCount::<Test>::get(2), 5);
		})
	}

	#[test]
	fn generate_groups_works() {
		new_test_ext().execute_with(|| {
			let block_hash = H256::from(&[1u8; 32]);
			frame_system::BlockHash::<Test>::insert(1, block_hash);

			let mut validators = Vec::new();
			for i in 0..7 {
				validators.push(pub_key_from_u64(i));
			}
			ValidatorsCollection::<Test>::set(validators);
			let groups: BTreeMap<u32, BTreeSet<TeaPubKey>> = Tea::generate_groups(&1);
			assert_eq!(groups.len(), 1);
			assert_eq!(groups[&0].len(), 7);

			let mut validators = Vec::new();
			for i in 0..11 {
				validators.push(pub_key_from_u64(i));
			}
			ValidatorsCollection::<Test>::set(validators);
			let groups: BTreeMap<u32, BTreeSet<TeaPubKey>> = Tea::generate_groups(&1);
			assert_eq!(groups.len(), 2);
			assert_eq!(groups[&0].len(), 6);
			assert_eq!(groups[&1].len(), 5);

			let mut validators = Vec::new();
			for i in 0..15 {
				validators.push(pub_key_from_u64(i));
			}
			ValidatorsCollection::<Test>::set(validators);
			let groups: BTreeMap<u32, BTreeSet<TeaPubKey>> = Tea::generate_groups(&1);
			assert_eq!(groups.len(), 2);
			assert_eq!(groups[&0].len(), 10);
			assert_eq!(groups[&1].len(), 5);

			let mut validators = Vec::new();
			for i in 0..21 {
				validators.push(pub_key_from_u64(i));
			}
			ValidatorsCollection::<Test>::set(validators);
			let groups: BTreeMap<u32, BTreeSet<TeaPubKey>> = Tea::generate_groups(&1);
			assert_eq!(groups.len(), 3);
			assert_eq!(groups[&0].len(), 10);
			assert_eq!(groups[&1].len(), 6);
			assert_eq!(groups[&2].len(), 5);
		})
	}

	fn pub_key_from_u64(value: u64) -> TeaPubKey {
		let mut hash_buf = [0u8; 32];
		hash_buf[24..32].copy_from_slice(&value.to_le_bytes());
		hash_buf
	}
}
