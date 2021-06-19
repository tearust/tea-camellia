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

		let remove_handler = |draw_box: &mut Option<Vec<u64>>| match draw_box {
			Some(draw_box) => {
				for id in draw_box.drain(0..) {
					CmlStore::<T>::remove(id);
				}
			}
			None => {}
		};
		LuckyDrawBox::<T>::mutate(CmlType::A, remove_handler);
		LuckyDrawBox::<T>::mutate(CmlType::B, remove_handler);
		LuckyDrawBox::<T>::mutate(CmlType::C, remove_handler);

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
		let mut draw_handler = |draw_box: &mut Option<Vec<u64>>,
		                        cml_type: CmlType,
		                        coupon_len: u32| match draw_box {
			Some(draw_box) => {
				for i in 0..coupon_len {
					ensure!(!draw_box.is_empty(), Error::<T>::NotEnoughDrawSeeds);

					let rand_index =
						Self::get_draw_seed_random_index(who, cml_type, i, draw_box.len() as u32);
					let seed_id = draw_box.swap_remove(rand_index as usize);
					seed_ids.push(seed_id);
				}
				Ok(())
			}
			None => Err(Error::<T>::DrawBoxNotInitialized),
		};

		LuckyDrawBox::<T>::mutate(CmlType::A, |a_box| {
			draw_handler(a_box, CmlType::A, a_coupon)
		})?;
		LuckyDrawBox::<T>::mutate(CmlType::B, |b_box| {
			draw_handler(b_box, CmlType::B, b_coupon)
		})?;
		LuckyDrawBox::<T>::mutate(CmlType::C, |c_box| {
			draw_handler(c_box, CmlType::C, c_coupon)
		})?;

		Ok(seed_ids)
	}

	fn get_draw_seed_random_index(
		who: &T::AccountId,
		cml_type: CmlType,
		index: u32,
		box_len: u32,
	) -> u32 {
		let mut salt = vec![cml_type as u8];
		salt.append(&mut index.to_le_bytes().to_vec());

		let rand_value = T::CommonUtils::generate_random(who.clone(), &salt);
		let (_, div_mod) = rand_value.div_mod(sp_core::U256::from(box_len));
		div_mod.as_u32()
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::new_test_ext;
	use crate::seeds::DefrostScheduleType;
	use crate::{mock::*, CmlStore, CmlType, LuckyDrawBox, Seed, SeedsCleaned, CML};

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

			LuckyDrawBox::<Test>::insert(CmlType::A, origin_a_box.clone());
			LuckyDrawBox::<Test>::insert(CmlType::B, origin_b_box.clone());
			LuckyDrawBox::<Test>::insert(CmlType::C, origin_c_box.clone());

			frame_system::Pallet::<Test>::set_block_number(100);
			let a_coupon = 2u32;
			let b_coupon = 3u32;
			let c_coupon = 4u32;
			let res = Cml::lucky_draw(&1, a_coupon, b_coupon, c_coupon);
			assert!(res.is_ok());

			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A).unwrap().len() as u32,
				10 - a_coupon
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B).unwrap().len() as u32,
				10 - b_coupon
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C).unwrap().len() as u32,
				10 - c_coupon
			);
			assert_eq!(
				res.clone().unwrap().len() as u32,
				a_coupon + b_coupon + c_coupon
			);
			println!("seeds are: {:?}", res.unwrap());
		})
	}

	#[test]
	fn try_clean_outdated_seeds_works() {
		new_test_ext().execute_with(|| {
			let origin_a_box: Vec<u64> = (1..=10).collect();
			let origin_b_box: Vec<u64> = (11..=20).collect();
			let origin_c_box: Vec<u64> = (21..=30).collect();

			LuckyDrawBox::<Test>::insert(CmlType::A, origin_a_box.clone());
			LuckyDrawBox::<Test>::insert(CmlType::B, origin_b_box.clone());
			LuckyDrawBox::<Test>::insert(CmlType::C, origin_c_box.clone());
			for id in origin_a_box.iter() {
				CmlStore::<Test>::insert(id, CML::new(default_seed()));
			}
			for id in origin_b_box.iter() {
				CmlStore::<Test>::insert(id, CML::new(default_seed()));
			}
			for id in origin_c_box.iter() {
				CmlStore::<Test>::insert(id, CML::new(default_seed()));
			}
			SeedsCleaned::<Test>::set(Some(false));

			Cml::try_clean_outdated_seeds((SEEDS_TIMEOUT_HEIGHT - 1) as u64);
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::A).unwrap().len(), 10); // not cleaned yet
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::B).unwrap().len(), 10); // not cleaned yet
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::C).unwrap().len(), 10); // not cleaned yet
			assert_eq!(SeedsCleaned::<Test>::get(), Some(false));

			Cml::try_clean_outdated_seeds(SEEDS_TIMEOUT_HEIGHT as u64);
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::A).unwrap().len(), 0);
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::B).unwrap().len(), 0);
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::C).unwrap().len(), 0);
			assert_eq!(SeedsCleaned::<Test>::get(), Some(true));
			for id in origin_a_box.iter() {
				assert!(!CmlStore::<Test>::contains_key(id));
			}
			for id in origin_b_box.iter() {
				assert!(!CmlStore::<Test>::contains_key(id));
			}
			for id in origin_c_box.iter() {
				assert!(!CmlStore::<Test>::contains_key(id));
			}
		})
	}

	fn default_seed() -> Seed {
		Seed {
			id: 0,
			cml_type: CmlType::A,
			defrost_schedule: DefrostScheduleType::Team,
			defrost_time: 0,
			lifespan: 0,
			performance: 0,
		}
	}
}
