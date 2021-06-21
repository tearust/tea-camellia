use super::*;
use sp_runtime::traits::AtLeast32BitUnsigned;

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
				*maybe_item = Some(Voucher { cml_type, amount });
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
		if SeedsCleaned::<T>::get() || block_number < T::TimoutHeight::get().into() {
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

		SeedsCleaned::<T>::set(true);
	}

	pub(crate) fn try_defrost_seeds(block_number: T::BlockNumber) -> Vec<CmlId> {
		let defrost_seeds: Vec<CmlId> = FrozenSeeds::<T>::iter()
			.filter(|(_, v)| block_number > *v)
			.map(|(k, _)| {
				CmlStore::<T>::mutate(k, |v| match v {
					Some(cml) => {
						cml.status = CmlStatus::SeedLive;
						Some(k)
					}
					None => None,
				})
			})
			.filter(|v| v.is_some())
			.map(|v| v.unwrap())
			.collect();
		defrost_seeds
			.iter()
			.for_each(|id| FrozenSeeds::<T>::remove(id));
		defrost_seeds
	}

	pub(crate) fn try_kill_cml(block_number: T::BlockNumber) -> Vec<CmlId> {
		let dead_cmls: Vec<CmlId> = CmlStore::<T>::iter()
			.filter(|(_, cml)| cml.status != CmlStatus::Dead && cml.should_death(block_number))
			.map(|(id, cml)| match cml.owner() {
				Some(owner) => {
					UserCmlStore::<T>::mutate(owner, |ids| {
						if let Some(ids) = ids {
							if let Some(index) = ids.iter().position(|v| *v == id) {
								ids.remove(index);
							}
						}
					});
					Some(id)
				}
				None => {
					None // should never happen
				}
			})
			.filter(|v| v.is_some())
			.map(|v| v.unwrap())
			.collect();
		dead_cmls.iter().for_each(|id| {
			CmlStore::<T>::remove(id);
		});
		dead_cmls
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

pub fn convert_seeds_to_cmls<AccountId, BlockNumber, Balance>(
	seeds: &Vec<Seed>,
) -> (
	Vec<CML<AccountId, BlockNumber, Balance>>,
	Vec<CmlId>,
	Vec<(CmlId, BlockNumber)>,
)
where
	AccountId: Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	let mut cml_list = Vec::new();
	let mut draw_box = Vec::new();
	let mut frozen_seeds = Vec::new();

	for seed in seeds {
		let mut cml = CML::new(seed.clone());
		if seed.defrost_time.is_zero() {
			cml.status = CmlStatus::SeedLive;
		} else {
			frozen_seeds.push((cml.id(), cml.intrinsic.defrost_time.into()));
		}

		cml_list.push(cml);
		draw_box.push(seed.id);
	}

	(cml_list, draw_box, frozen_seeds)
}

#[cfg(test)]
mod tests {
	use crate::mock::new_test_ext;
	use crate::seeds::DefrostScheduleType;
	use crate::{
		mock::*, CmlId, CmlStatus, CmlStore, CmlType, FrozenSeeds, LuckyDrawBox, Seed,
		SeedsCleaned, StakingCategory, StakingItem, UserCmlStore, CML,
	};
	use rand::{thread_rng, Rng};

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
			SeedsCleaned::<Test>::set(false);

			Cml::try_clean_outdated_seeds((SEEDS_TIMEOUT_HEIGHT - 1) as u64);
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::A).unwrap().len(), 10); // not cleaned yet
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::B).unwrap().len(), 10); // not cleaned yet
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::C).unwrap().len(), 10); // not cleaned yet
			assert_eq!(SeedsCleaned::<Test>::get(), false);

			Cml::try_clean_outdated_seeds(SEEDS_TIMEOUT_HEIGHT as u64);
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::A).unwrap().len(), 0);
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::B).unwrap().len(), 0);
			assert_eq!(LuckyDrawBox::<Test>::get(CmlType::C).unwrap().len(), 0);
			assert_eq!(SeedsCleaned::<Test>::get(), true);
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

	#[test]
	fn try_defrost_seeds_works() {
		new_test_ext().execute_with(|| {
			const SEEDS_COUNT: usize = 100;
			const START_HEIGHT: u64 = 1;
			const STOP_HEIGHT: u64 = 100;

			let mut rng = thread_rng();
			for i in 0..SEEDS_COUNT {
				let frozen_time = rng.gen_range(START_HEIGHT..STOP_HEIGHT) as u32;
				CmlStore::<Test>::insert(i as CmlId, CML::new(seed_from_defrost_type(frozen_time)));
				FrozenSeeds::<Test>::insert(i as CmlId, frozen_time as u64);
			}

			for i in START_HEIGHT..=STOP_HEIGHT {
				let count_before = FrozenSeeds::<Test>::iter().count();
				let defrosted_seeds = Cml::try_defrost_seeds(i);
				for id in defrosted_seeds.iter() {
					assert_eq!(
						CmlStore::<Test>::get(id).unwrap().status,
						CmlStatus::SeedLive
					);
				}
				let count_after = FrozenSeeds::<Test>::iter().count();
				assert_eq!(count_before, defrosted_seeds.len() + count_after);
			}

			assert_eq!(0, FrozenSeeds::<Test>::iter().count());
		})
	}

	#[test]
	fn try_kill_cml_works() {
		new_test_ext().execute_with(|| {
			const SEEDS_COUNT: usize = 100;
			const START_HEIGHT: u64 = 1;
			const STOP_HEIGHT: u64 = 100;
			const START_USER_ID: u64 = 1;
			const STOP_USER_ID: u64 = 10;

			let mut rng = thread_rng();
			for i in 0..SEEDS_COUNT {
				let user_id = rng.gen_range(START_USER_ID..STOP_USER_ID);
				let plant_time = rng.gen_range(START_HEIGHT..STOP_HEIGHT);
				let lifespan = rng.gen_range(START_HEIGHT..STOP_HEIGHT) as u32;

				let mut cml = CML::new(seed_from_lifespan(lifespan));
				cml.planted_at = plant_time;
				cml.staking_slot.push(StakingItem {
					owner: user_id,
					category: StakingCategory::Cml,
					amount: None,
					cml: None,
				});

				CmlStore::<Test>::insert(i as CmlId, cml);
				UserCmlStore::<Test>::mutate(user_id, |ids| match ids {
					Some(ids) => ids.push(i as CmlId),
					None => *ids = Some(vec![i as CmlId]),
				});
			}

			for i in START_HEIGHT..=(STOP_HEIGHT * 2) {
				let count_before = CmlStore::<Test>::iter().count();
				let dead_cmls = Cml::try_kill_cml(i);
				for id in dead_cmls.iter() {
					assert!(!CmlStore::<Test>::contains_key(id));
				}
				let count_after = CmlStore::<Test>::iter().count();
				assert_eq!(count_before, dead_cmls.len() + count_after);
			}

			assert_eq!(0, CmlStore::<Test>::iter().count());
			for i in START_USER_ID..=STOP_USER_ID {
				let cml_list = UserCmlStore::<Test>::get(i);
				assert!(cml_list.is_none() || cml_list.unwrap().is_empty());
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

	fn seed_from_defrost_type(defrost_time: u32) -> Seed {
		let mut seed = default_seed();
		seed.defrost_time = defrost_time;
		seed
	}

	fn seed_from_lifespan(lifespan: u32) -> Seed {
		let mut seed = default_seed();
		seed.lifespan = lifespan;
		seed
	}
}
