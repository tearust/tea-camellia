use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub fn next_id() -> CmlId {
		LastCmlId::<T>::mutate(|id| {
			if *id < u64::MAX {
				*id += 1;
			} else {
				*id = 1;
			}

			*id
		})
	}

	pub fn check_seed_validity(cml_id: CmlId, height: &T::BlockNumber) -> DispatchResult {
		let cml = CmlStore::<T>::get(cml_id);
		ensure!(cml.is_seed(), Error::<T>::CmlIsNotSeed);
		cml.check_seed_validity(height)
			.map_err(|e| Error::<T>::from(e))?;

		Ok(())
	}

	pub fn add_or_create_coupon(
		who: &T::AccountId,
		cml_type: CmlType,
		schedule_type: DefrostScheduleType,
		amount: u32,
	) {
		let set_store_hanlder = |maybe_item: &mut Option<Coupon>| {
			if let Some(ref mut item) = maybe_item {
				item.amount = amount;
			} else {
				*maybe_item = Some(Coupon { cml_type, amount });
			}
		};
		match schedule_type {
			DefrostScheduleType::Investor => {
				InvestorCouponStore::<T>::mutate(&who, cml_type, set_store_hanlder)
			}
			DefrostScheduleType::Team => {
				TeamCouponStore::<T>::mutate(&who, cml_type, set_store_hanlder)
			}
		}
	}

	pub(crate) fn is_coupon_outdated(block_number: T::BlockNumber) -> bool {
		block_number > T::CouponTimoutHeight::get()
	}

	pub(crate) fn try_clean_outdated_coupons(block_number: T::BlockNumber) {
		if !Self::is_coupon_outdated(block_number) {
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

		InvestorCouponStore::<T>::remove_all(None);
		TeamCouponStore::<T>::remove_all(None);
	}

	pub(crate) fn try_kill_cml(block_number: T::BlockNumber) -> Vec<CmlId> {
		let dead_cmls: Vec<CmlId> = CmlStore::<T>::iter()
			.filter(|(_, cml)| {
				if cml.is_seed() {
					cml.is_fresh_seed() && cml.check_seed_validity(&block_number).is_err()
				} else {
					cml.check_tree_validity(&block_number).is_err()
				}
			})
			.map(|(id, cml)| {
				Self::clean_cml_related(&cml);
				id
			})
			.collect();
		dead_cmls.iter().for_each(|id| {
			CmlStore::<T>::remove(id);
		});
		dead_cmls
	}

	pub(crate) fn stop_mining_inner(who: &T::AccountId, cml_id: CmlId, machine_id: &MachineId) {
		CmlStore::<T>::mutate(cml_id, |cml| {
			let staking_slots_length = cml.staking_slots().len();
			// user reverse order iterator to avoid staking index adjustments
			for i in (1..staking_slots_length).rev() {
				if let Some(staking_item) = cml.staking_slots().get(i) {
					Self::unstake(
						&staking_item.owner.clone(),
						cml,
						i as u32,
						T::StakingPrice::get(),
					);
				}
			}

			// unstake the first slot
			Self::unstake(who, cml, 0, T::StakingPrice::get());
			cml.stop_mining();
		});
		MinerItemStore::<T>::remove(machine_id);
	}

	pub(crate) fn customer_staking_length(
		owner: &T::AccountId,
		cml: &CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>,
	) -> u32 {
		let mut length = 0;
		for staking_item in cml.staking_slots() {
			if staking_item.owner.eq(owner) {
				continue;
			}

			if let Some(cml_id) = staking_item.cml {
				length += CmlStore::<T>::get(cml_id).staking_weight();
			} else {
				length += 1;
			}
		}

		length
	}

	pub(crate) fn pay_for_miner_customer(owner: &T::AccountId, cml_id: CmlId) {
		let cml = CmlStore::<T>::get(cml_id);
		for staking_item in cml.staking_slots() {
			if staking_item.owner.eq(owner) {
				continue;
			}

			let punishment = if let Some(cml_id) = staking_item.cml {
				T::StopMiningPunishment::get() * CmlStore::<T>::get(cml_id).staking_weight().into()
			} else {
				T::StopMiningPunishment::get()
			};

			if let Err(e) = T::CurrencyOperations::transfer(
				owner,
				&staking_item.owner,
				punishment,
				ExistenceRequirement::AllowDeath,
			) {
				// see https://github.com/tearust/tea-camellia/issues/13
				log::error!("pay for miner failed: {:?}", e);
				return;
			}
		}
	}

	fn clean_cml_related(
		cml: &CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>,
	) {
		// clean mining related
		if let Some(machine_id) = cml.machine_id() {
			Self::stop_mining_inner(
				cml.owner().unwrap_or(&Default::default()),
				cml.id(),
				machine_id,
			);
		}
		// clean staking related
		if let Some((cml_id, staking_index)) = cml.staking_index() {
			if CmlStore::<T>::contains_key(cml_id) {
				CmlStore::<T>::mutate(cml_id, |cml| {
					let index = staking_index as usize;
					if cml.staking_slots().len() <= index {
						return;
					}
					cml.staking_slots_mut().remove(index);
					Self::adjust_staking_cml_index(cml, staking_index);
				});
			}
		}
		// clean user cml store
		if let Some(owner) = cml.owner() {
			UserCmlStore::<T>::remove(owner, cml.id());
		}
	}

	pub(crate) fn fetch_coupons(
		who: &T::AccountId,
		schedule_type: DefrostScheduleType,
		remove: bool,
	) -> (u32, u32, u32) {
		let get_coupon_amount = |cml_type: CmlType, who: &T::AccountId| {
			let item = match schedule_type {
				DefrostScheduleType::Investor => {
					if remove {
						InvestorCouponStore::<T>::take(who, cml_type)
					} else {
						InvestorCouponStore::<T>::get(who, cml_type)
					}
				}
				DefrostScheduleType::Team => {
					if remove {
						TeamCouponStore::<T>::take(who, cml_type)
					} else {
						TeamCouponStore::<T>::get(who, cml_type)
					}
				}
			};
			match item {
				Some(coupon) => coupon.amount,
				None => 0,
			}
		};

		(
			get_coupon_amount(CmlType::A, who),
			get_coupon_amount(CmlType::B, who),
			get_coupon_amount(CmlType::C, who),
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

		let rand_value =
			sp_core::U256::from(T::CommonUtils::generate_random(who.clone(), &salt).as_bytes());
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
		unstake_balance: BalanceOf<T>,
	) -> bool {
		if let Some(staking_item) = staking_to.staking_slots().get(staking_index as usize) {
			let (index, is_balance_staking) = match staking_item.cml {
				Some(cml_id) => (
					CmlStore::<T>::mutate(cml_id, |cml| staking_to.unstake(None, Some(cml))),
					false,
				),
				None => {
					T::CurrencyOperations::unreserve(&who, unstake_balance);
					(
						staking_to.unstake::<CML<
							T::AccountId,
							T::BlockNumber,
							BalanceOf<T>,
							T::SeedFreshDuration,
						>>(Some(staking_index), None),
						true,
					)
				}
			};
			if let Some(index) = index {
				Self::adjust_staking_cml_index(staking_to, index);
			}
			return is_balance_staking;
		}
		true
	}

	pub(crate) fn check_miner_ip_validity(miner_ip: &Vec<u8>) -> DispatchResult {
		ensure!(!miner_ip.is_empty(), Error::<T>::InvalidMinerIp);
		Ok(())
	}

	pub(crate) fn adjust_staking_cml_index(
		staking_to: &mut CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>,
		staking_index: StakingIndex,
	) {
		let index = staking_index as usize;
		for i in index..staking_to.staking_slots().len() {
			if let Some(staking_item) = staking_to.staking_slots().get(i) {
				if let Some(cml_id) = staking_item.cml {
					if !CmlStore::<T>::contains_key(cml_id) {
						continue; // should never happen
					}
					CmlStore::<T>::mutate(cml_id, |cml| {
						cml.shift_staking_index(i as StakingIndex);
					});
				}
			}
		}
	}
}

impl<T: cml::Config> Task for cml::Pallet<T> {
	/// Called after a miner has complete a RA task.
	fn complete_ra_task(machine_id: MachineId, task_point: ServiceTaskPoint) {
		// for now all ra task will have one unit point
		let machine_item = MinerItemStore::<T>::get(machine_id);

		if MiningCmlTaskPoints::<T>::contains_key(machine_item.cml_id) {
			MiningCmlTaskPoints::<T>::mutate(machine_item.cml_id, |point| {
				*point = point.saturating_add(task_point);
			});
		} else {
			MiningCmlTaskPoints::<T>::insert(machine_item.cml_id, task_point);
		}
	}
}

pub fn init_from_genesis_coupons<T>(genesis_coupons: &GenesisCoupons<T::AccountId>)
where
	T: Config,
{
	genesis_coupons.coupons.iter().for_each(|coupon_config| {
		let coupon: Coupon = coupon_config.clone().into();
		match coupon_config.schedule_type {
			DefrostScheduleType::Investor => InvestorCouponStore::<T>::insert(
				&coupon_config.account,
				coupon_config.cml_type,
				coupon,
			),
			DefrostScheduleType::Team => {
				TeamCouponStore::<T>::insert(&coupon_config.account, coupon_config.cml_type, coupon)
			}
		}
	});
}

pub fn init_from_genesis_seeds<T>(genesis_seeds: &GenesisSeeds)
where
	T: Config,
{
	let (a_cml_list, investor_a_draw_box, team_a_draw_box) = convert_genesis_seeds_to_cmls::<
		T::AccountId,
		T::BlockNumber,
		BalanceOf<T>,
		T::SeedFreshDuration,
	>(&genesis_seeds.a_seeds);
	let (b_cml_list, investor_b_draw_box, team_b_draw_box) = convert_genesis_seeds_to_cmls::<
		T::AccountId,
		T::BlockNumber,
		BalanceOf<T>,
		T::SeedFreshDuration,
	>(&genesis_seeds.b_seeds);
	let (c_cml_list, investor_c_draw_box, team_c_draw_box) = convert_genesis_seeds_to_cmls::<
		T::AccountId,
		T::BlockNumber,
		BalanceOf<T>,
		T::SeedFreshDuration,
	>(&genesis_seeds.c_seeds);
	LuckyDrawBox::<T>::insert(
		CmlType::A,
		DefrostScheduleType::Investor,
		investor_a_draw_box,
	);
	LuckyDrawBox::<T>::insert(CmlType::A, DefrostScheduleType::Team, team_a_draw_box);
	LuckyDrawBox::<T>::insert(
		CmlType::B,
		DefrostScheduleType::Investor,
		investor_b_draw_box,
	);
	LuckyDrawBox::<T>::insert(CmlType::B, DefrostScheduleType::Team, team_b_draw_box);
	LuckyDrawBox::<T>::insert(
		CmlType::C,
		DefrostScheduleType::Investor,
		investor_c_draw_box,
	);
	LuckyDrawBox::<T>::insert(CmlType::C, DefrostScheduleType::Team, team_c_draw_box);

	a_cml_list
		.into_iter()
		.chain(b_cml_list.into_iter())
		.chain(c_cml_list.into_iter())
		.for_each(|cml| CmlStore::<T>::insert(cml.id(), cml));

	LastCmlId::<T>::set(
		(genesis_seeds.a_seeds.len() + genesis_seeds.b_seeds.len() + genesis_seeds.c_seeds.len())
			as CmlId,
	)
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
	FreshDuration: Get<BlockNumber> + TypeInfo,
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
	use crate::functions::{init_from_genesis_coupons, init_from_genesis_seeds};
	use crate::generator::init_genesis;
	use crate::param::{
		GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT, TEAM_PERCENTAGE,
	};
	use crate::{
		mock::*, CmlId, CmlStore, CmlType, CouponConfig, DefrostScheduleType, GenesisCoupons,
		InvestorCouponStore, LastCmlId, LuckyDrawBox, MinerItemStore, Seed, SeedProperties,
		StakingProperties, TeamCouponStore, TreeProperties, UserCmlStore, CML,
	};
	use frame_support::{assert_ok, traits::Currency};
	use rand::{thread_rng, Rng};

	#[test]
	fn init_from_genesis_coupons_works() {
		new_test_ext().execute_with(|| {
			let user11 = 11;
			let user12 = 12;
			let user13 = 13;
			let user21 = 21;
			let user22 = 22;
			let user23 = 23;
			let genesis_coupons = GenesisCoupons {
				coupons: vec![
					CouponConfig {
						account: user11,
						cml_type: CmlType::A,
						schedule_type: DefrostScheduleType::Team,
						amount: 1,
					},
					CouponConfig {
						account: user12,
						cml_type: CmlType::B,
						schedule_type: DefrostScheduleType::Team,
						amount: 2,
					},
					CouponConfig {
						account: user13,
						cml_type: CmlType::C,
						schedule_type: DefrostScheduleType::Team,
						amount: 3,
					},
					CouponConfig {
						account: user21,
						cml_type: CmlType::A,
						schedule_type: DefrostScheduleType::Investor,
						amount: 4,
					},
					CouponConfig {
						account: user22,
						cml_type: CmlType::B,
						schedule_type: DefrostScheduleType::Investor,
						amount: 5,
					},
					CouponConfig {
						account: user23,
						cml_type: CmlType::C,
						schedule_type: DefrostScheduleType::Investor,
						amount: 6,
					},
				],
			};
			init_from_genesis_coupons::<Test>(&genesis_coupons);

			assert_eq!(InvestorCouponStore::<Test>::iter().count(), 3);
			assert_eq!(TeamCouponStore::<Test>::iter().count(), 3);

			assert_eq!(
				TeamCouponStore::<Test>::get(user11, CmlType::A)
					.unwrap()
					.amount,
				1
			);
			assert_eq!(
				TeamCouponStore::<Test>::get(user12, CmlType::B)
					.unwrap()
					.amount,
				2
			);
			assert_eq!(
				TeamCouponStore::<Test>::get(user13, CmlType::C)
					.unwrap()
					.amount,
				3
			);
			assert_eq!(
				InvestorCouponStore::<Test>::get(user21, CmlType::A)
					.unwrap()
					.amount,
				4
			);
			assert_eq!(
				InvestorCouponStore::<Test>::get(user22, CmlType::B)
					.unwrap()
					.amount,
				5
			);
			assert_eq!(
				InvestorCouponStore::<Test>::get(user23, CmlType::C)
					.unwrap()
					.amount,
				6
			);
		})
	}

	#[test]
	fn init_from_genesis_seeds_works() {
		new_test_ext().execute_with(|| {
			let genesis_seeds = init_genesis([1; 32]);
			init_from_genesis_seeds::<Test>(&genesis_seeds);

			assert_eq!(
				CmlStore::<Test>::iter().count() as u64,
				GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT
			);
			assert_eq!(
				LastCmlId::<Test>::get(),
				GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Team).len() as u64,
				GENESIS_SEED_A_COUNT * TEAM_PERCENTAGE / 100
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Investor).len() as u64,
				GENESIS_SEED_A_COUNT * (100 - TEAM_PERCENTAGE) / 100
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Team).len() as u64,
				GENESIS_SEED_B_COUNT * TEAM_PERCENTAGE / 100
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Investor).len() as u64,
				GENESIS_SEED_B_COUNT * (100 - TEAM_PERCENTAGE) / 100
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Team).len() as u64,
				GENESIS_SEED_C_COUNT * TEAM_PERCENTAGE / 100
			);
			assert_eq!(
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Investor).len() as u64,
				GENESIS_SEED_C_COUNT * (100 - TEAM_PERCENTAGE) / 100
			);
		})
	}

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
	fn try_clean_outdated_coupons_works() {
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

			Cml::try_clean_outdated_coupons(SEEDS_TIMEOUT_HEIGHT as u64);
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

			Cml::try_clean_outdated_coupons(SEEDS_TIMEOUT_HEIGHT as u64 + 1);
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

			assert_eq!(InvestorCouponStore::<Test>::iter().count(), 0);
			assert_eq!(TeamCouponStore::<Test>::iter().count(), 0);
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

	#[test]
	fn try_kill_cml_works_with_fresh_seed() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let user3 = 3;
			let cml1_id = 1;
			let cml2_id = 2;
			let cml3_id = 3;

			UserCmlStore::<Test>::insert(user1, cml1_id, ());
			let mut cml1 =
				CML::from_genesis_seed(seed_from_lifespan(cml1_id, 100, DefrostScheduleType::Team));
			cml1.set_owner(&user1);
			cml1.defrost(&0);
			CmlStore::<Test>::insert(cml1_id, cml1);

			UserCmlStore::<Test>::insert(user2, cml2_id, ());
			let mut cml2 =
				CML::from_genesis_seed(seed_from_lifespan(cml2_id, 200, DefrostScheduleType::Team));
			cml2.set_owner(&user2);
			cml2.defrost(&100);
			CmlStore::<Test>::insert(cml2_id, cml2);

			// cml3_id is frozen seed that should not be killed whenever
			UserCmlStore::<Test>::insert(user3, cml3_id, ());
			let mut cml3 =
				CML::from_genesis_seed(seed_from_lifespan(cml3_id, 100, DefrostScheduleType::Team));
			cml3.set_owner(&user3);
			CmlStore::<Test>::insert(cml3_id, cml3);

			let dead_cmls = Cml::try_kill_cml(99);
			assert_eq!(dead_cmls.len(), 0);

			let dead_cmls = Cml::try_kill_cml(SEED_FRESH_DURATION as u64);
			assert_eq!(dead_cmls.len(), 1);
			assert_eq!(dead_cmls[0], cml1_id);
			assert!(!CmlStore::<Test>::contains_key(cml1_id));
			assert!(!UserCmlStore::<Test>::contains_key(user1, cml1_id));

			let dead_cmls = Cml::try_kill_cml(SEED_FRESH_DURATION as u64 + 100);
			assert_eq!(dead_cmls.len(), 1);
			assert_eq!(dead_cmls[0], cml2_id);
			assert!(!CmlStore::<Test>::contains_key(cml2_id));
			assert!(!UserCmlStore::<Test>::contains_key(user2, cml2_id));

			assert_eq!(CmlStore::<Test>::iter().count(), 1);
			assert_eq!(UserCmlStore::<Test>::iter().count(), 1);
		})
	}

	#[test]
	fn try_kill_cml_works_with_mining() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let cml1_id = 1;
			let cml2_id = 2;
			let machine1_id = [1; 32];
			let machine2_id = [2; 32];

			UserCmlStore::<Test>::insert(user1, cml1_id, ());
			let mut cml1 =
				CML::from_genesis_seed(seed_from_lifespan(cml1_id, 100, DefrostScheduleType::Team));
			cml1.set_owner(&user1);
			CmlStore::<Test>::insert(cml1_id, cml1);
			<Test as crate::Config>::Currency::make_free_balance_be(&user1, STAKING_PRICE);
			assert_ok!(Cml::start_mining(
				Origin::signed(user1),
				cml1_id,
				machine1_id,
				b"machine1 ip".to_vec(),
				b"orbitdb id".to_vec(),
			));

			UserCmlStore::<Test>::insert(user2, cml2_id, ());
			let mut cml2 =
				CML::from_genesis_seed(seed_from_lifespan(cml2_id, 200, DefrostScheduleType::Team));
			cml2.set_owner(&user2);
			CmlStore::<Test>::insert(cml2_id, cml2);
			<Test as crate::Config>::Currency::make_free_balance_be(&user2, STAKING_PRICE);
			assert_ok!(Cml::start_mining(
				Origin::signed(user2),
				cml2_id,
				machine2_id,
				b"machine2 ip".to_vec(),
				b"orbitdb id".to_vec(),
			));

			let dead_cmls = Cml::try_kill_cml(99);
			assert_eq!(dead_cmls.len(), 0);

			let dead_cmls = Cml::try_kill_cml(100);
			assert_eq!(dead_cmls.len(), 1);
			assert_eq!(dead_cmls[0], cml1_id);
			assert!(!MinerItemStore::<Test>::contains_key(&machine1_id));
			assert!(!CmlStore::<Test>::contains_key(cml1_id));
			assert!(!UserCmlStore::<Test>::contains_key(user1, cml1_id));

			let dead_cmls = Cml::try_kill_cml(200);
			assert_eq!(dead_cmls.len(), 1);
			assert_eq!(dead_cmls[0], cml2_id);
			assert!(!MinerItemStore::<Test>::contains_key(&machine2_id));
			assert!(!CmlStore::<Test>::contains_key(cml2_id));
			assert!(!UserCmlStore::<Test>::contains_key(user2, cml2_id));

			assert_eq!(MinerItemStore::<Test>::iter().count(), 0);
			assert_eq!(CmlStore::<Test>::iter().count(), 0);
			assert_eq!(UserCmlStore::<Test>::iter().count(), 0);
		})
	}

	#[test]
	fn try_kill_cml_works_with_staking() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let user3 = 3;
			let cml1_id = 1;
			let cml2_id = 2;
			let cml3_id = 3;
			let machine3_id = [3; 32];

			UserCmlStore::<Test>::insert(user1, cml1_id, ());
			let mut cml1 =
				CML::from_genesis_seed(seed_from_lifespan(cml1_id, 100, DefrostScheduleType::Team));
			cml1.set_owner(&user1);
			CmlStore::<Test>::insert(cml1_id, cml1);

			UserCmlStore::<Test>::insert(user2, cml2_id, ());
			let mut cml2 =
				CML::from_genesis_seed(seed_from_lifespan(cml2_id, 200, DefrostScheduleType::Team));
			cml2.set_owner(&user2);
			CmlStore::<Test>::insert(cml2_id, cml2);

			UserCmlStore::<Test>::insert(user3, cml3_id, ());
			let mut cml3 =
				CML::from_genesis_seed(seed_from_lifespan(cml3_id, 300, DefrostScheduleType::Team));
			cml3.set_owner(&user3);
			CmlStore::<Test>::insert(cml3_id, cml3);
			<Test as crate::Config>::Currency::make_free_balance_be(&user3, STAKING_PRICE);
			assert_ok!(Cml::start_mining(
				Origin::signed(user3),
				cml3_id,
				machine3_id,
				b"machine3 ip".to_vec(),
				b"orbitdb id".to_vec(),
			));

			assert_ok!(Cml::start_staking(
				Origin::signed(user1),
				cml3_id,
				Some(cml1_id),
				None,
			));
			assert_ok!(Cml::start_staking(
				Origin::signed(user2),
				cml3_id,
				Some(cml2_id),
				None,
			));
			let staking_slots = CmlStore::<Test>::get(cml3_id).staking_slots().clone();
			assert_eq!(staking_slots.len(), 3);
			assert_eq!(staking_slots[0].owner, user3);
			assert_eq!(staking_slots[1].owner, user1);
			assert_eq!(staking_slots[2].owner, user2);

			let dead_cmls = Cml::try_kill_cml(100);
			assert_eq!(dead_cmls.len(), 1);
			assert_eq!(dead_cmls[0], cml1_id);
			assert!(!CmlStore::<Test>::contains_key(cml1_id));
			assert!(!UserCmlStore::<Test>::contains_key(user1, cml1_id));
			let staking_slots = CmlStore::<Test>::get(cml3_id).staking_slots().clone();
			assert_eq!(staking_slots.len(), 2);
			assert_eq!(staking_slots[0].owner, user3);
			assert_eq!(staking_slots[1].owner, user2);

			let dead_cmls = Cml::try_kill_cml(200);
			assert_eq!(dead_cmls.len(), 1);
			assert_eq!(dead_cmls[0], cml2_id);
			assert!(!CmlStore::<Test>::contains_key(cml2_id));
			assert!(!UserCmlStore::<Test>::contains_key(user2, cml2_id));
			let staking_slots = CmlStore::<Test>::get(cml3_id).staking_slots().clone();
			assert_eq!(staking_slots.len(), 1);
			assert_eq!(staking_slots[0].owner, user3);
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
