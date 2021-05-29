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

		CmlStore::<T>::insert(cml.id, cml.clone());

		if UserCmlStore::<T>::contains_key(&who) {
      let mut list = UserCmlStore::<T>::take(&who).unwrap();
      list.insert(0, cml.id);
      UserCmlStore::<T>::insert(&who, list);
    } 
    else {
      UserCmlStore::<T>::insert(&who, vec![cml.id]);
    }
	}

	pub fn remove_cml_by_id() {}

	// fn get_cml_id_list_by_account(
	// 	who: &T::AccountId,
	// ) -> Vec<T::AssetId> {
	// 	let list = {
	// 		if <UserCmlStore<T>>::contains_key(&who) {
	// 			UserCmlStore::<T>::get(&who).unwrap()
	// 		}
	// 		else {
	// 			vec![]
	// 		}
	// 	};
		
	// 	list
	// }

	pub fn update_cml(
		cml: CML<T::AssetId, T::AccountId, T::BlockNumber>,
	) {
		CmlStore::<T>::mutate(cml.id, |maybe_item| {	
			if let Some(ref mut item) = maybe_item {
				*item = cml;
			}
		});
	}

	pub fn get_cml_by_id(
		cml_id: &T::AssetId
	) -> Result<CML<T::AssetId, T::AccountId, T::BlockNumber>, Error<T>> {
		let cml = CmlStore::<T>::get(&cml_id).ok_or(Error::<T>::NotFoundCML)?;

		Ok(cml)
	}

	pub fn update_cml_to_active(
		cml_id: &T::AssetId,
		miner_id: Vec<u8>,
		staking_item: StakingItem<T::AccountId, T::AssetId>,
	) -> Result<(), Error<T>> {

		let mut cml = Self::get_cml_by_id(&cml_id)?;
		cml.status = b"CML_Live".to_vec();
		cml.miner_id = miner_id;
		cml.staking_slot.push(staking_item);

		Self::update_cml(cml);

		Ok(())
	}

	pub fn staking_to_cml(
		staking_item: StakingItem<T::AccountId, T::AssetId>,
		target_cml_id: &T::AssetId,
	) -> Result<(), Error<T>> {
		let mut cml = CmlStore::<T>::get(&target_cml_id).ok_or(Error::<T>::NotFoundCML)?;

		if cml.status != b"CML_Live".to_vec() {
			return Err(Error::<T>::CMLNotLive);
		}

		cml.staking_slot.push(staking_item);

		Self::update_cml(cml.clone());

		Ok(())
	}
}
