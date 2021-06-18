use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub fn get_random_life(voucher_group: CmlType) -> T::BlockNumber {
		// TODO random

		let life: u64 = match voucher_group {
			CmlType::A => 40_000_000,
			CmlType::B => 20_000_000,
			CmlType::C => 10_000_000,
		};

		life.saturated_into::<T::BlockNumber>()
	}

	pub fn get_random_mining_rate(voucher_group: CmlType) -> u8 {
		// TODO random

		let rate: u8 = match voucher_group {
			CmlType::A => 40,
			CmlType::B => 20,
			CmlType::C => 10,
		};

		rate
	}

	pub fn get_next_id() -> CmlId {
		let cid = LastCmlId::<T>::get();
		LastCmlId::<T>::mutate(|id| *id += 1);

		cid
	}

	pub fn set_voucher(who: &T::AccountId, cml_type: CmlType, amount: u32) {
		UserVoucherStore::<T>::mutate(&who, cml_type, |maybe_item| {
			if let Some(ref mut item) = maybe_item {
				item.amount = amount;
			} else {
				// Run here means not from genesis block, so no lock amount and unlock type.
				*maybe_item = Some(Voucher {
					cml_type,
					amount,
					lock: None,
					unlock_type: None,
				});
			}
		});
	}

	pub fn add_cml(who: &T::AccountId, cml: CML<T::AccountId, T::BlockNumber, BalanceOf<T>>) {
		CmlStore::<T>::insert(cml.id(), cml.clone());

		if UserCmlStore::<T>::contains_key(&who) {
			let mut list = UserCmlStore::<T>::take(&who).unwrap();
			list.insert(0, cml.id());
			UserCmlStore::<T>::insert(&who, list);
		} else {
			UserCmlStore::<T>::insert(&who, vec![cml.id()]);
		}
	}

	pub fn remove_cml_by_id() {}

	pub fn update_cml(cml: CML<T::AccountId, T::BlockNumber, BalanceOf<T>>) {
		CmlStore::<T>::mutate(cml.id(), |maybe_item| {
			if let Some(ref mut item) = maybe_item {
				*item = cml;
			}
		});
	}

	pub fn get_cml_by_id(
		cml_id: &CmlId,
	) -> Result<CML<T::AccountId, T::BlockNumber, BalanceOf<T>>, Error<T>> {
		let cml = CmlStore::<T>::get(&cml_id).ok_or(Error::<T>::NotFoundCML)?;

		Ok(cml)
	}

	pub fn check_belongs(cml_id: &CmlId, who: &T::AccountId) -> Result<(), Error<T>> {
		let user_cml = UserCmlStore::<T>::get(&who).ok_or(Error::<T>::CMLOwnerInvalid)?;
		if !user_cml.contains(&cml_id) {
			return Err(Error::<T>::CMLOwnerInvalid);
		}

		Ok(())
	}

	pub fn update_cml_to_active(
		cml_id: &CmlId,
		machine_id: MachineId,
		staking_item: StakingItem<T::AccountId, CmlId, BalanceOf<T>>,
		block_number: T::BlockNumber,
	) -> Result<(), Error<T>> {
		let mut cml = Self::get_cml_by_id(&cml_id)?;
		cml.status = CmlStatus::CmlLive;
		cml.machine_id = Some(machine_id);
		cml.staking_slot.push(staking_item);
		cml.planted_at = block_number;

		Self::update_cml(cml);

		Ok(())
	}

	pub fn staking_to_cml(
		staking_item: StakingItem<T::AccountId, CmlId, BalanceOf<T>>,
		target_cml_id: &CmlId,
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
		cml_id: &CmlId,
		target_account: &T::AccountId,
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
			} else {
				*maybe_list = Some(vec![*cml_id]);
			}
		});

		Ok(())
	}

	pub(crate) fn try_clean_outdated_seeds(block_number: T::BlockNumber) {
		if block_number < T::TimoutHeight::get().into() || SeedsCleaned::<T>::get().unwrap_or(false)
		{
			return;
		}

		// todo remove updated cmls
		// Seeds::<T>::remove_all();
		SeedsCleaned::<T>::set(Some(true));
	}

	pub(crate) fn take_vouchers(who: &T::AccountId) -> Vec<Voucher> {
		let mut voucher_list: Vec<Voucher> = Vec::new();

		let type_list = vec![CmlType::A, CmlType::B, CmlType::C];
		for ty in type_list.iter() {
			if let Some(voucher) = UserVoucherStore::<T>::take(who, ty) {
				voucher_list.push(voucher);
			}
		}

		voucher_list
	}
}
