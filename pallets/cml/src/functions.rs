use super::*;

impl<T: cml::Config> CmlOperation for cml::Pallet<T> {
	type AccountId = T::AccountId;
	type Balance = BalanceOf<T>;
	type BlockNumber = T::BlockNumber;
	type FreshDuration = T::SeedFreshDuration;

	fn get_cml_by_id(
		cml_id: &CmlId,
	) -> Result<
		CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
		DispatchError,
	> {
		ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
		Ok(CmlStore::<T>::get(cml_id))
	}

	fn check_belongs(cml_id: &u64, who: &Self::AccountId) -> DispatchResult {
		ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
		ensure!(
			UserCmlStore::<T>::contains_key(who, cml_id),
			Error::<T>::CMLOwnerInvalid
		);
		Ok(())
	}

	fn transfer_cml_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	) -> DispatchResult {
		Self::check_belongs(cml_id, from_account)?;

		CmlStore::<T>::mutate(&cml_id, |cml| -> DispatchResult {
			if cml.is_mining() {
				// todo check balance before create_balance_staking
				let staking_item =
					Self::create_balance_staking(target_account, T::StakingPrice::get())?;
				cml.swap_first_slot(staking_item);

				// TODO balance
			}
			Ok(())
		})?;

		// remove from from UserCmlStore
		UserCmlStore::<T>::remove(from_account, cml_id);
		UserCmlStore::<T>::insert(target_account, cml_id, ());

		Ok(())
	}
}

impl<T: cml::Config> cml::Pallet<T> {
	pub fn get_next_id() -> CmlId {
		let cid = LastCmlId::<T>::get();
		LastCmlId::<T>::mutate(|id| *id += 1);

		cid
	}

	pub fn check_seed_validity(cml_id: CmlId, height: &T::BlockNumber) -> DispatchResult {
		let cml = CmlStore::<T>::get(cml_id);
		ensure!(cml.seed_valid(height), Error::<T>::SeedNotValid);

		Ok(())
	}

	pub fn set_voucher(
		who: &T::AccountId,
		cml_type: CmlType,
		schedule_type: DefrostScheduleType,
		amount: u32,
	) {
		let set_store_hanlder = |maybe_item: &mut Option<Voucher>| {
			if let Some(ref mut item) = maybe_item {
				item.amount = amount;
			} else {
				*maybe_item = Some(Voucher { cml_type, amount });
			}
		};
		match schedule_type {
			DefrostScheduleType::Investor => {
				InvestorVoucherStore::<T>::mutate(&who, cml_type, set_store_hanlder)
			}
			DefrostScheduleType::Team => {
				TeamVoucherStore::<T>::mutate(&who, cml_type, set_store_hanlder)
			}
		}
	}

	pub fn add_cml(
		who: &T::AccountId,
		cml: CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>,
	) {
		let cml_id = cml.id();
		CmlStore::<T>::insert(cml_id, cml);
		UserCmlStore::<T>::insert(who, cml_id, ());
	}

	pub(crate) fn is_voucher_outdated(block_number: T::BlockNumber) -> bool {
		block_number > T::VoucherTimoutHeight::get()
	}

	pub(crate) fn try_clean_outdated_vouchers(block_number: T::BlockNumber) {
		if !Self::is_voucher_outdated(block_number) {
			return;
		}

		let remove_handler = |draw_box: &mut Vec<u64>| {
			for id in draw_box.drain(0..) {
				CmlStore::<T>::remove(id);
			}
		};
		LuckyDrawBox::<T>::mutate(CmlType::A, DefrostScheduleType::Investor, remove_handler);
		LuckyDrawBox::<T>::mutate(CmlType::A, DefrostScheduleType::Team, remove_handler);
		LuckyDrawBox::<T>::mutate(CmlType::B, DefrostScheduleType::Investor, remove_handler);
		LuckyDrawBox::<T>::mutate(CmlType::B, DefrostScheduleType::Team, remove_handler);
		LuckyDrawBox::<T>::mutate(CmlType::C, DefrostScheduleType::Investor, remove_handler);
		LuckyDrawBox::<T>::mutate(CmlType::C, DefrostScheduleType::Team, remove_handler);

		InvestorVoucherStore::<T>::remove_all();
		TeamVoucherStore::<T>::remove_all();
	}

	pub(crate) fn try_kill_cml(block_number: T::BlockNumber) -> Vec<CmlId> {
		let dead_cmls: Vec<CmlId> = CmlStore::<T>::iter()
			// todo change should_dead error handling method if needed later
			.filter(|(_, cml)| cml.should_dead(&block_number))
			.map(|(id, cml)| match cml.owner() {
				Some(owner) => {
					UserCmlStore::<T>::remove(owner, id);
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

	pub(crate) fn take_vouchers(
		who: &T::AccountId,
		schedule_type: DefrostScheduleType,
	) -> (u32, u32, u32) {
		let get_voucher_amount = |cml_type: CmlType, who: &T::AccountId| {
			let item = match schedule_type {
				DefrostScheduleType::Investor => InvestorVoucherStore::<T>::take(who, cml_type),
				DefrostScheduleType::Team => TeamVoucherStore::<T>::take(who, cml_type),
			};
			match item {
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

	pub(crate) fn lucky_draw_box_all_empty(schedule_types: Vec<DefrostScheduleType>) -> bool {
		let cml_types = vec![CmlType::A, CmlType::B, CmlType::C];
		for st in schedule_types.iter() {
			for ct in cml_types.iter() {
				if LuckyDrawBox::<T>::contains_key(ct.clone(), st.clone())
					&& !LuckyDrawBox::<T>::get(ct.clone(), st.clone()).is_empty()
				{
					return false;
				}
			}
		}
		return true;
	}

	pub(crate) fn check_luck_draw(
		a_coupon: u32,
		b_coupon: u32,
		c_coupon: u32,
		schedule_type: DefrostScheduleType,
	) -> DispatchResult {
		ensure!(
			LuckyDrawBox::<T>::contains_key(CmlType::A, schedule_type)
				&& LuckyDrawBox::<T>::get(CmlType::A, schedule_type).len() >= a_coupon as usize,
			Error::<T>::NotEnoughDrawSeeds
		);
		ensure!(
			LuckyDrawBox::<T>::contains_key(CmlType::B, schedule_type)
				&& LuckyDrawBox::<T>::get(CmlType::B, schedule_type).len() >= b_coupon as usize,
			Error::<T>::NotEnoughDrawSeeds
		);
		ensure!(
			LuckyDrawBox::<T>::contains_key(CmlType::C, schedule_type)
				&& LuckyDrawBox::<T>::get(CmlType::C, schedule_type).len() >= c_coupon as usize,
			Error::<T>::NotEnoughDrawSeeds
		);

		Ok(())
	}

	pub(crate) fn lucky_draw(
		who: &T::AccountId,
		a_coupon: u32,
		b_coupon: u32,
		c_coupon: u32,
		schedule_type: DefrostScheduleType,
	) -> Vec<CmlId> {
		let mut seed_ids = Vec::new();
		let mut draw_handler = |draw_box: &mut Vec<u64>, cml_type: CmlType, coupon_len: u32| {
			for i in 0..coupon_len {
				let rand_index =
					Self::get_draw_seed_random_index(who, cml_type, i, draw_box.len() as u32);
				let seed_id = draw_box.swap_remove(rand_index as usize);
				seed_ids.push(seed_id);
			}
		};

		LuckyDrawBox::<T>::mutate(CmlType::A, schedule_type, |a_box| {
			draw_handler(a_box, CmlType::A, a_coupon);
		});
		LuckyDrawBox::<T>::mutate(CmlType::B, schedule_type, |b_box| {
			draw_handler(b_box, CmlType::B, b_coupon);
		});
		LuckyDrawBox::<T>::mutate(CmlType::C, schedule_type, |c_box| {
			draw_handler(c_box, CmlType::C, c_coupon);
		});

		seed_ids
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

	pub(crate) fn stake(
		who: &T::AccountId,
		staking_to: &mut CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>,
		staking_cml: &Option<CmlId>,
		current_height: &T::BlockNumber,
	) -> Option<StakingIndex> {
		match staking_cml {
			Some(cml_id) => CmlStore::<T>::mutate(cml_id, |cml| -> Option<StakingIndex> {
				staking_to.stake(&who, &current_height, None, Some(cml))
			}),
			None => {
				T::CurrencyOperations::reserve(&who, T::StakingPrice::get()).unwrap();
				staking_to
					.stake::<CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>>(
						&who,
						current_height,
						Some(T::StakingPrice::get()),
						None,
					)
			}
		}
	}

	pub(crate) fn unstake(
		who: &T::AccountId,
		staking_to: &mut CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>,
		staking_index: StakingIndex,
	) {
		if let Some(staking_item) = staking_to.staking_slots().get(staking_index as usize) {
			match staking_item.cml {
				Some(cml_id) => CmlStore::<T>::mutate(cml_id, |cml| {
					staking_to.unstake(None, Some(cml));
				}),
				None => {
					T::CurrencyOperations::unreserve(&who, T::StakingPrice::get()).unwrap();
					staking_to
						.unstake::<CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>>(
							Some(staking_index),
							None,
						)
				}
			};
		}
	}
}

pub fn convert_genesis_seeds_to_cmls<AccountId, BlockNumber, Balance, FreshDuration>(
	seeds: &Vec<Seed>,
) -> (
	Vec<CML<AccountId, BlockNumber, Balance, FreshDuration>>,
	Vec<CmlId>,
	Vec<CmlId>,
)
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	FreshDuration: Get<BlockNumber>,
{
	let mut cml_list = Vec::new();
	let mut investor_draw_box = Vec::new();
	let mut team_draw_box = Vec::new();

	for seed in seeds {
		let cml = CML::from_genesis_seed(seed.clone());

		cml_list.push(cml);
		match seed.defrost_schedule.unwrap() {
			DefrostScheduleType::Investor => investor_draw_box.push(seed.id),
			DefrostScheduleType::Team => team_draw_box.push(seed.id),
		}
	}

	(cml_list, investor_draw_box, team_draw_box)
}

#[cfg(test)]
mod tests {
	use crate::{
		mock::*, CmlId, CmlStore, CmlType, DefrostScheduleType, InvestorVoucherStore, LuckyDrawBox,
		Seed, SeedProperties, TeamVoucherStore, TreeProperties, UserCmlStore, CML,
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
		// investor lucky draw works
		new_test_ext().execute_with(|| {
			let origin_a_box: Vec<u64> = (1..=10).collect();
			let origin_b_box: Vec<u64> = (11..=20).collect();
			let origin_c_box: Vec<u64> = (21..=30).collect();

			LuckyDrawBox::<Test>::insert(
				CmlType::A,
				DefrostScheduleType::Investor,
				origin_a_box.clone(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::B,
				DefrostScheduleType::Investor,
				origin_b_box.clone(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::C,
				DefrostScheduleType::Investor,
				origin_c_box.clone(),
			);

			frame_system::Pallet::<Test>::set_block_number(100);
			let a_coupon = 2u32;
			let b_coupon = 3u32;
			let c_coupon = 4u32;
			assert!(Cml::check_luck_draw(
				a_coupon,
				b_coupon,
				c_coupon,
				DefrostScheduleType::Investor
			)
			.is_ok());
			let res = Cml::lucky_draw(
				&1,
				a_coupon,
				b_coupon,
				c_coupon,
				DefrostScheduleType::Investor,
			);

			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Investor).len() as u32,
				10 - a_coupon
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Investor).len() as u32,
				10 - b_coupon
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Investor).len() as u32,
				10 - c_coupon
			);
			assert_eq!(res.len() as u32, a_coupon + b_coupon + c_coupon);
			println!("seeds are: {:?}", res);
		});

		// team lucky draw works
		new_test_ext().execute_with(|| {
			let origin_a_box: Vec<u64> = (1..=10).collect();
			let origin_b_box: Vec<u64> = (11..=20).collect();
			let origin_c_box: Vec<u64> = (21..=30).collect();

			LuckyDrawBox::<Test>::insert(
				CmlType::A,
				DefrostScheduleType::Team,
				origin_a_box.clone(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::B,
				DefrostScheduleType::Team,
				origin_b_box.clone(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::C,
				DefrostScheduleType::Team,
				origin_c_box.clone(),
			);

			frame_system::Pallet::<Test>::set_block_number(100);
			let a_coupon = 2u32;
			let b_coupon = 3u32;
			let c_coupon = 4u32;
			assert!(
				Cml::check_luck_draw(a_coupon, b_coupon, c_coupon, DefrostScheduleType::Team)
					.is_ok()
			);
			let res = Cml::lucky_draw(&1, a_coupon, b_coupon, c_coupon, DefrostScheduleType::Team);

			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Team).len() as u32,
				10 - a_coupon
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Team).len() as u32,
				10 - b_coupon
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Team).len() as u32,
				10 - c_coupon
			);
			assert_eq!(res.len() as u32, a_coupon + b_coupon + c_coupon);
			println!("seeds are: {:?}", res);
		});
	}

	#[test]
	fn draw_to_the_last_works() {
		new_test_ext().execute_with(|| {
			LuckyDrawBox::<Test>::insert(
				CmlType::A,
				DefrostScheduleType::Team,
				(1..=10).collect::<Vec<u64>>(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::B,
				DefrostScheduleType::Team,
				(11..=20).collect::<Vec<u64>>(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::C,
				DefrostScheduleType::Team,
				(21..=30).collect::<Vec<u64>>(),
			);

			frame_system::Pallet::<Test>::set_block_number(100);
			let a_coupon = 10u32;
			let b_coupon = 10u32;
			let c_coupon = 10u32;
			assert!(
				Cml::check_luck_draw(a_coupon, b_coupon, c_coupon, DefrostScheduleType::Team)
					.is_ok()
			);
			let res = Cml::lucky_draw(&1, a_coupon, b_coupon, c_coupon, DefrostScheduleType::Team);

			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Team).len() as u32,
				0
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Team).len() as u32,
				0
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Team).len() as u32,
				0
			);
			assert_eq!(res.len() as u32, a_coupon + b_coupon + c_coupon);
		})
	}

	#[test]
	fn try_clean_outdated_vouchers_works() {
		new_test_ext().execute_with(|| {
			let origin_investor_a_box: Vec<u64> = (1..=10).collect();
			let origin_investor_b_box: Vec<u64> = (11..=20).collect();
			let origin_investor_c_box: Vec<u64> = (21..=30).collect();
			let origin_team_a_box: Vec<u64> = (31..=40).collect();
			let origin_team_b_box: Vec<u64> = (41..=50).collect();
			let origin_team_c_box: Vec<u64> = (51..=60).collect();

			LuckyDrawBox::<Test>::insert(
				CmlType::A,
				DefrostScheduleType::Investor,
				origin_investor_a_box.clone(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::B,
				DefrostScheduleType::Investor,
				origin_investor_b_box.clone(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::C,
				DefrostScheduleType::Investor,
				origin_investor_c_box.clone(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::A,
				DefrostScheduleType::Team,
				origin_team_a_box.clone(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::B,
				DefrostScheduleType::Team,
				origin_team_b_box.clone(),
			);
			LuckyDrawBox::<Test>::insert(
				CmlType::C,
				DefrostScheduleType::Team,
				origin_team_c_box.clone(),
			);
			for id in origin_investor_a_box.iter() {
				CmlStore::<Test>::insert(
					id,
					CML::from_genesis_seed(default_genesis_seed(DefrostScheduleType::Investor)),
				);
			}
			for id in origin_investor_b_box.iter() {
				CmlStore::<Test>::insert(
					id,
					CML::from_genesis_seed(default_genesis_seed(DefrostScheduleType::Investor)),
				);
			}
			for id in origin_investor_c_box.iter() {
				CmlStore::<Test>::insert(
					id,
					CML::from_genesis_seed(default_genesis_seed(DefrostScheduleType::Investor)),
				);
			}
			for id in origin_team_a_box.iter() {
				CmlStore::<Test>::insert(
					id,
					CML::from_genesis_seed(default_genesis_seed(DefrostScheduleType::Team)),
				);
			}
			for id in origin_team_b_box.iter() {
				CmlStore::<Test>::insert(
					id,
					CML::from_genesis_seed(default_genesis_seed(DefrostScheduleType::Team)),
				);
			}
			for id in origin_team_c_box.iter() {
				CmlStore::<Test>::insert(
					id,
					CML::from_genesis_seed(default_genesis_seed(DefrostScheduleType::Team)),
				);
			}

			Cml::try_clean_outdated_vouchers(SEEDS_TIMEOUT_HEIGHT as u64);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Investor).len(),
				10
			); // not cleaned yet
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Investor).len(),
				10
			); // not cleaned yet
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Investor).len(),
				10
			); // not cleaned yet
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Team).len(),
				10
			); // not cleaned yet
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Team).len(),
				10
			); // not cleaned yet
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Team).len(),
				10
			); // not cleaned yet

			Cml::try_clean_outdated_vouchers(SEEDS_TIMEOUT_HEIGHT as u64 + 1);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Investor).len(),
				0
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Investor).len(),
				0
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Investor).len(),
				0
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Team).len(),
				0
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Team).len(),
				0
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Team).len(),
				0
			);

			for id in origin_investor_a_box.iter() {
				assert!(!CmlStore::<Test>::contains_key(id));
			}
			for id in origin_investor_b_box.iter() {
				assert!(!CmlStore::<Test>::contains_key(id));
			}
			for id in origin_investor_c_box.iter() {
				assert!(!CmlStore::<Test>::contains_key(id));
			}
			for id in origin_team_a_box.iter() {
				assert!(!CmlStore::<Test>::contains_key(id));
			}
			for id in origin_team_b_box.iter() {
				assert!(!CmlStore::<Test>::contains_key(id));
			}
			for id in origin_team_c_box.iter() {
				assert!(!CmlStore::<Test>::contains_key(id));
			}

			assert_eq!(InvestorVoucherStore::<Test>::iter().count(), 0);
			assert_eq!(TeamVoucherStore::<Test>::iter().count(), 0);
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

				let mut cml = CML::from_genesis_seed(seed_from_lifespan(
					i as u64,
					lifespan,
					DefrostScheduleType::Team,
				));
				cml.defrost(&0);
				cml.set_owner(&user_id);
				cml.convert_to_tree(&plant_time);

				CmlStore::<Test>::insert(cml.id(), cml);
				UserCmlStore::<Test>::insert(user_id, i as CmlId, ());
			}

			CmlStore::<Test>::iter().for_each(|(_, cml)| {
				assert!(cml.should_dead(&(STOP_HEIGHT * 2)));
			});

			for i in START_HEIGHT..=(STOP_HEIGHT * 2) {
				let count_before = CmlStore::<Test>::iter().count();
				let dead_cmls = Cml::try_kill_cml(i);
				for id in dead_cmls.iter() {
					assert!(!CmlStore::<Test>::contains_key(id));
				}
				let count_after = CmlStore::<Test>::iter().count();
				assert_eq!(count_before, dead_cmls.len() + count_after);
			}

			assert_eq!(CmlStore::<Test>::iter().count(), 0);
			assert_eq!(UserCmlStore::<Test>::iter().count(), 0);
		})
	}

	fn default_genesis_seed(schedule_type: DefrostScheduleType) -> Seed {
		Seed {
			id: 0,
			cml_type: CmlType::A,
			defrost_schedule: Some(schedule_type),
			defrost_time: Some(0),
			lifespan: 0,
			performance: 0,
		}
	}

	fn seed_from_lifespan(id: CmlId, lifespan: u32, schedule_type: DefrostScheduleType) -> Seed {
		let mut seed = default_genesis_seed(schedule_type);
		seed.id = id;
		seed.lifespan = lifespan;
		seed
	}
}
