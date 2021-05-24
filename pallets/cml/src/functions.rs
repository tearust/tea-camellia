use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub fn get_random_life() -> T::BlockNumber {
		let life: u64 = 20_000_000;
		life.saturated_into::<T::BlockNumber>()
	}

	pub fn get_random_mining_rate() -> u8 {
		10 as u8
	}

	pub fn get_next_id() -> T::AssetId {
		let cid = LastAssetId::<T>::get();
		let _id = cid.clone();
		LastAssetId::<T>::mutate(|_id| *_id += One::one());

		cid
	}

	pub fn new_cml_from_dai(
		group: Vec<u8>,
		status: Vec<u8>,  // Seed_Live, Seed_Frozen
	) -> CML<T::AssetId, T::AccountId, T::BlockNumber> {

		// life time, lock time
		let current_block = frame_system::Pallet::<T>::block_number();
		let life_time = current_block + Self::get_random_life();
		let lock_time = <T::BlockNumber>::saturated_from(0_u64);
		
		CML {
			id: Self::get_next_id(),
			group,
			status,
			mining_rate: Self::get_random_mining_rate(),
			life_time,
			lock_time,
			staking_slot: vec![],
			created_at: current_block,
			miner_id: b"".to_vec(),
		}

  }

	pub fn get_dai(who: &T::AccountId) -> Dai {
    match DaiStore::<T>::get(&who) {
			Some(n) => n,
			None => 0 as Dai,
		}
  }

  pub fn set_dai(
    who: &T::AccountId,
    amount: Dai
  ) {
    DaiStore::<T>::mutate(&who, |n| *n = Some(amount));
	}

	pub fn add_cml(
		who: &T::AccountId,
		cml: CML<T::AssetId, T::AccountId, T::BlockNumber>,
	) {
		if CmlStore::<T>::contains_key(&who) {
      let mut list = CmlStore::<T>::take(&who).unwrap();
      list.insert(0, cml);
      CmlStore::<T>::insert(&who, list);
    } 
    else {
      CmlStore::<T>::insert(&who, vec![cml]);
    }
	}

	pub fn remove_cml_by_id() {}

	fn get_cml_list_by_account(
		who: &T::AccountId,
	) -> Vec<CML<T::AssetId, T::AccountId, T::BlockNumber>> {
		let list = {
			if <CmlStore<T>>::contains_key(&who) {
				CmlStore::<T>::get(&who).unwrap()
			}
			else {
				vec![]
			}
		};
		
		list
	}

	pub fn set_cml_by_index(
		who: &T::AccountId,
		cml: CML<T::AssetId, T::AccountId, T::BlockNumber>,
		index: usize,
	) {
		CmlStore::<T>::mutate(&who, |maybe_list| {	
			if let Some(ref mut list) = maybe_list {
				list.remove(index);
				list.insert(index, cml);
			}
		});
	}

	pub fn find_cml_index(
		who: &T::AccountId,
		cml_id: &T::AssetId,
	) -> (Vec<CML<T::AssetId, T::AccountId, T::BlockNumber>>, i32) {
		let list = Self::get_cml_list_by_account(&who);

		let index = match list.iter().position(|cml| cml.id == *cml_id) {
			Some(i) => i as i32,
			None => -1,
		};

		(list, index)
	}

	pub fn update_cml_to_active(
		who: &T::AccountId,
		cml_id: &T::AssetId,
		miner_id: Vec<u8>,
		staking_item: StakingItem<T::AccountId, T::AssetId>,
	) -> Result<(), Error<T>> {
		let (mut list, index) = Self::find_cml_index(&who, &cml_id);

		if index < 0 {
			return Err(Error::<T>::NotFoundCML);
		}

		let cml: &mut CML<T::AssetId, T::AccountId, T::BlockNumber> = list.get_mut(index as usize).unwrap();

		cml.status = b"CML_Live".to_vec();
		cml.miner_id = miner_id;

		cml.staking_slot.push(staking_item);

		Self::set_cml_by_index(&who, cml.clone(), index as usize);

		Ok(())
	}

	pub fn staking_to_cml(
		staking_item: StakingItem<T::AccountId, T::AssetId>,

		who: &T::AccountId,
		target_cml_id: &T::AssetId,
	) -> Result<(), Error<T>> {
		let (mut list, index) = Self::find_cml_index(&who, &target_cml_id);

		if index < 0 {
			return Err(Error::<T>::NotFoundCML);
		}

		let cml: &mut CML<T::AssetId, T::AccountId, T::BlockNumber> = list.get_mut(index as usize).unwrap();

		if cml.status != b"CML_Live".to_vec() {
			return Err(Error::<T>::CMLNotLive);
		}

		cml.staking_slot.push(staking_item);

		Self::set_cml_by_index(&who, cml.clone(), index as usize);
		
		Ok(())
	}
}
