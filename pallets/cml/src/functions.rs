use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub fn get_random_life(
		voucher_group: VoucherGroup,
	) -> T::BlockNumber {
		// TODO random

		let life: u64 = match voucher_group {
			VoucherGroup::A => 40_000_000,
			VoucherGroup::B => 20_000_000,
			VoucherGroup::C => 10_000_000,
		};

		life.saturated_into::<T::BlockNumber>()
	}

	pub fn get_random_mining_rate(
		voucher_group: VoucherGroup,
	) -> u8 {
		// TODO random

		let rate: u8 = match voucher_group {
			VoucherGroup::A => 40,
			VoucherGroup::B => 20,
			VoucherGroup::C => 10,
		};

		rate
	}

	pub fn get_next_id() -> T::CmlId {
		let cid = LastCmlId::<T>::get();
		let _id = cid.clone();
		LastCmlId::<T>::mutate(|_id| *_id += One::one());

		cid
	}

	fn new_one_cml_by_voucher(
		group: CmlGroup,
		voucher_group: VoucherGroup,
	) -> CML<T::CmlId, T::AccountId, T::BlockNumber, BalanceOf<T>> {

		// life time, lock time
		let current_block = frame_system::Pallet::<T>::block_number();
		let life_time = current_block + Self::get_random_life(voucher_group);
		let lock_time = <T::BlockNumber>::saturated_from(0_u64);  //TODO random
		
		CML {
			id: Self::get_next_id(),
			group,
			status: CmlStatus::SeedFrozen,
			mining_rate: Self::get_random_mining_rate(voucher_group),
			life_time,
			lock_time,
			staking_slot: vec![],
			created_at: current_block,
			miner_id: b"".to_vec(),
		}

  }

	pub fn new_cml_from_voucher(
		group: CmlGroup,
		voucher_amount: u32,
		voucher_group: VoucherGroup,
	) -> Vec<CML<T::CmlId, T::AccountId, T::BlockNumber, BalanceOf<T>>> {
		let mut list = vec![];

		let mut i = 0;
		while i < voucher_amount {
			let cml = Self::new_one_cml_by_voucher(group, voucher_group);
			list.push(cml);
			i += 1;
		}

		list
  }

  pub fn set_voucher(
		who: &T::AccountId,
		group: VoucherGroup,
    amount: u32
  ) {
    UserVoucherStore::<T>::mutate(&who, group, |maybe_item| {
			if let Some(ref mut item) = maybe_item {
				item.amount = amount;
			}
			else {
				// Run here means not from genesis block, so no lock amount and unlock type.
				*maybe_item = Some(Voucher {
					group,
					amount,
					lock: None,
					unlock_type: None,
				});
			}
		});
	}

	pub fn add_cml(
		who: &T::AccountId,
		cml: CML<T::CmlId, T::AccountId, T::BlockNumber, BalanceOf<T>>,
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


	pub fn update_cml(
		cml: CML<T::CmlId, T::AccountId, T::BlockNumber, BalanceOf<T>>,
	) {
		CmlStore::<T>::mutate(cml.id, |maybe_item| {	
			if let Some(ref mut item) = maybe_item {
				*item = cml;
			}
		});
	}

	pub fn get_cml_by_id(
		cml_id: &T::CmlId
	) -> Result<CML<T::CmlId, T::AccountId, T::BlockNumber, BalanceOf<T>>, Error<T>> {
		let cml = CmlStore::<T>::get(&cml_id).ok_or(Error::<T>::NotFoundCML)?;

		Ok(cml)
	}

	pub fn check_belongs(
		cml_id: &T::CmlId,
		who: &T::AccountId,
	) -> Result<(), Error<T>>{
		let user_cml = UserCmlStore::<T>::get(&who).ok_or(Error::<T>::CMLOwnerInvalid)?;
		if !user_cml.contains(&cml_id) {
			return Err(Error::<T>::CMLOwnerInvalid);
		}

		Ok(())
	}

	pub fn update_cml_to_active(
		cml_id: &T::CmlId,
		miner_id: Vec<u8>,
		staking_item: StakingItem<T::AccountId, T::CmlId, BalanceOf<T>>,
	) -> Result<(), Error<T>> {

		let mut cml = Self::get_cml_by_id(&cml_id)?;
		cml.status = CmlStatus::CmlLive;
		cml.miner_id = miner_id;
		cml.staking_slot.push(staking_item);

		Self::update_cml(cml);

		Ok(())
	}

	pub fn staking_to_cml(
		staking_item: StakingItem<T::AccountId, T::CmlId, BalanceOf<T>>,
		target_cml_id: &T::CmlId,
	) -> Result<(), Error<T>> {
		let mut cml = CmlStore::<T>::get(&target_cml_id).ok_or(Error::<T>::NotFoundCML)?;

		if cml.status != CmlStatus::CmlLive {
			return Err(Error::<T>::CMLNotLive);
		}

		cml.staking_slot.push(staking_item);

		Self::update_cml(cml.clone());

		Ok(())
	}


	pub fn transfer_cml_other(
		from_account: &T::AccountId,
		cml_id: &T::CmlId,
		target_account:  &T::AccountId,
	) -> Result<(), Error<T>> {
		let mut cml = CmlStore::<T>::get(&cml_id).ok_or(Error::<T>::NotFoundCML)?;

		let user_cml = UserCmlStore::<T>::get(&from_account).ok_or(Error::<T>::CMLOwnerInvalid)?;
		let from_index = match user_cml.iter().position(|x| *x == *cml_id) {
			Some(index) => index,
			None => {
				return Err(Error::<T>::CMLOwnerInvalid);
			}
		};

		if cml.status == CmlStatus::CmlLive {
			let staking_item = StakingItem {
				owner: target_account.clone(),
				category: StakingCategory::Tea,
				amount: Some(T::StakingPrice::get()),
				cml: None,
			};
			cml.staking_slot.remove(0);
			cml.staking_slot.insert(0, staking_item);

			Self::update_cml(cml);

			// TODO balance
		}
		

		// remove from from UserCmlStore
		UserCmlStore::<T>::mutate(&from_account, |maybe_list| {
			if let Some(ref mut list) = maybe_list {
				list.remove(from_index);
			}
		});

		// add to target UserCmlStore
		UserCmlStore::<T>::mutate(&target_account, |maybe_list| {
			if let Some(ref mut list) = maybe_list {
				list.push(*cml_id);
			}
			else {
				*maybe_list = Some(vec![*cml_id]);
			}
		});
			
		Ok(())
	}
}
