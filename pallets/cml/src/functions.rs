use super::*;

impl<T: cml::Config> cml::Pallet<T> {
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

	pub(crate) fn take_vouchers(who: &T::AccountId) -> (u32, u32, u32) {
		let get_voucher_amount = |cml_type: CmlType, who: &T::AccountId| {
			match UserVoucherStore::<T>::take(who, cml_type) {
				Some(voucher) => voucher.amount,
				None => 0,
			}
		};

		(
			get_voucher_amount(CmlType::A, who),
			get_voucher_amount(CmlType::B, who),
			get_voucher_amount(CmlType::C, who),
		)
	}

	pub(crate) fn lucky_draw(
		who: &T::AccountId,
		a_coupon: u32,
		b_coupon: u32,
		c_coupon: u32,
	) -> Result<Vec<CmlId>, DispatchError> {
		let mut seed_ids = Vec::new();
		let mut draw_handler = |draw_box: &mut Option<Vec<u64>>, coupon_len: u32| match draw_box {
			Some(draw_box) => {
				for i in 0..coupon_len {
					ensure!(!draw_box.is_empty(), Error::<T>::NotEnoughDrawSeeds);

					let rand_index =
						Self::get_draw_seed_random_index(who, i, draw_box.len() as u32);
					let seed_id = draw_box.swap_remove(rand_index as usize);
					seed_ids.push(seed_id);
				}
				Ok(())
			}
			None => Err(Error::<T>::DrawBoxNotInitialized),
		};

		TypeALuckyDrawBox::<T>::mutate(|a_box| draw_handler(a_box, a_coupon))?;
		TypeBLuckyDrawBox::<T>::mutate(|b_box| draw_handler(b_box, b_coupon))?;
		TypeCLuckyDrawBox::<T>::mutate(|c_box| draw_handler(c_box, c_coupon))?;

		Ok(seed_ids)
	}

	fn get_draw_seed_random_index(who: &T::AccountId, index: u32, box_len: u32) -> u32 {
		let rand_value =
			T::CommonUtils::generate_random(who.clone(), &index.to_le_bytes().to_vec());
		let (_, div_mod) = rand_value.div_mod(sp_core::U256::from(box_len));
		div_mod.as_u32()
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::new_test_ext;
	use crate::{mock::*, TypeALuckyDrawBox, TypeBLuckyDrawBox, TypeCLuckyDrawBox};

	#[test]
	fn div_mod_works() {
		let a = sp_core::U256::from(30u32);
		let b = sp_core::U256::from(7u32);
		let (c, d) = a.div_mod(b);
		assert_eq!(c, sp_core::U256::from(4));
		assert_eq!(d, sp_core::U256::from(2));
	}

	#[test]
	fn lucky_draw_works() {
		new_test_ext().execute_with(|| {
			let origin_a_box: Vec<u64> = (1..=10).collect();
			let origin_b_box: Vec<u64> = (11..=20).collect();
			let origin_c_box: Vec<u64> = (21..=30).collect();

			TypeALuckyDrawBox::<Test>::set(Some(origin_a_box.clone()));
			TypeBLuckyDrawBox::<Test>::set(Some(origin_b_box.clone()));
			TypeCLuckyDrawBox::<Test>::set(Some(origin_c_box.clone()));

			frame_system::Pallet::<Test>::set_block_number(100);
			let a_coupon = 2u32;
			let b_coupon = 3u32;
			let c_coupon = 4u32;
			let res = Cml::lucky_draw(&1, a_coupon, b_coupon, c_coupon);
			assert!(res.is_ok());

			assert_eq!(
				TypeALuckyDrawBox::<Test>::get().unwrap().len() as u32,
				10 - a_coupon
			);
			assert_eq!(
				TypeBLuckyDrawBox::<Test>::get().unwrap().len() as u32,
				10 - b_coupon
			);
			assert_eq!(
				TypeCLuckyDrawBox::<Test>::get().unwrap().len() as u32,
				10 - c_coupon
			);
			assert_eq!(
				res.clone().unwrap().len() as u32,
				a_coupon + b_coupon + c_coupon
			);
			println!("seeds are: {:?}", res.unwrap());
		})
	}
}
