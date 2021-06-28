use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub fn get_user_cml_list(who: &T::AccountId) -> Vec<u64> {
		UserCmlStore::<T>::iter()
			.filter(|(user, _, _)| *user == *who)
			.map(|(_, id, _)| id)
			.collect()
	}

	pub fn get_user_staking_list(who: &T::AccountId) -> Vec<(u64, u64)> {
		let mut result = Vec::new();
		for (_, miner_item) in MinerItemStore::<T>::iter() {
			let cml = CmlStore::<T>::get(miner_item.cml_id);
			if cml.is_none() {
				continue;
			}

			let cml_id = cml.as_ref().unwrap().id();
			let mut staking_list: Vec<(u64, u64)> = cml
				.unwrap()
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
}
