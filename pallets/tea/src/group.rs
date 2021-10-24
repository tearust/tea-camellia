use super::*;

impl<T: tea::Config> tea::Pallet<T> {
	pub(crate) fn should_update_validators(n: &T::BlockNumber) -> bool {
		*n % T::UpdateValidatorsDuration::get() == 0u32.into()
	}

	pub(crate) fn update_runtime_status(block_number: T::BlockNumber) {
		for (tea_id, mut node) in Nodes::<T>::iter() {
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
	}

	pub(crate) fn generate_groups(block_number: T::BlockNumber) -> BTreeMap<u32, Vec<TeaPubKey>> {
		if !frame_system::BlockHash::<T>::contains_key(&block_number) {
			return Default::default();
		}

		let validators_count = ValidatorsCollection::<T>::get().len() as u32;
		let group_count = validators_count / T::MaxGroupMemberCount::get() + 1;
		let last_group_count = validators_count % T::MaxGroupMemberCount::get();
		let last_group_insufficient_number = last_group_count < T::MinGroupMemberCount::get();

		let block_hash = frame_system::BlockHash::<T>::get(&block_number);
		let mut tea_id_hash_numbers = Vec::new();
		ValidatorsCollection::<T>::get().iter().for_each(|tea_id| {
			tea_id_hash_numbers.push((tea_id.clone(), Self::hash_number(&block_hash, tea_id)));
		});

		tea_id_hash_numbers.sort_by(|(_, a), (_, b)| a.cmp(b));
		let sorted_tea_ids: Vec<TeaPubKey> = tea_id_hash_numbers
			.into_iter()
			.map(|(tea_id, _)| tea_id)
			.collect();

		let mut general_groups: BTreeMap<u32, Vec<TeaPubKey>> = BTreeMap::new();
		(0..group_count).into_iter().for_each(|id| {
			general_groups.insert(id, Default::default());
		});

		let mut current_group_index = 0u32;
		for i in 0..validators_count {
			if let Some(array) = general_groups.get_mut(&current_group_index) {
				array.push(sorted_tea_ids[i as usize]);
			}

			if (last_group_insufficient_number
				&& validators_count - i - 1 <= T::MinGroupMemberCount::get())
				|| ((i + 1) % group_count == 0)
			{
				current_group_index += 1;
			}
		}

		general_groups
	}

	pub(crate) fn hash_number(block_hash: &T::Hash, tea_id: &TeaPubKey) -> u64 {
		let payload = (block_hash, tea_id);
		let hash: H256 = payload.using_encoded(blake2_256).into();
		hash.to_low_u64_le()
	}
}
