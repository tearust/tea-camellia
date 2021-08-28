use super::*;

impl<T: genesis_exchange::Config> MiningOperation for genesis_exchange::Pallet<T> {
	type AccountId = T::AccountId;

	fn check_buying_mining_machine(who: &Self::AccountId, cml_id: u64) -> DispatchResult {
		let cost = Self::machine_cost_by_id(cml_id)?;
		ensure!(
			USDStore::<T>::get(who) >= cost,
			Error::<T>::InsufficientUSDToPayMiningMachineCost
		);
		Ok(())
	}

	fn buy_mining_machine(who: &Self::AccountId, cml_id: u64) {
		if let Err(_e) = Self::check_buying_mining_machine(who, cml_id) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			return;
		}

		match Self::machine_cost_by_id(cml_id) {
			Ok(cost) => {
				USDStore::<T>::mutate(who, |balance| *balance = balance.saturating_sub(cost))
			}
			Err(e) => {
				// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
				log::error!(
					"get mining cml (id: {}) machine cost error: {:?}",
					cml_id,
					e
				);
			}
		}
	}

	fn check_redeem_coupons(
		who: &Self::AccountId,
		a_coupon: u32,
		b_coupon: u32,
		c_coupon: u32,
	) -> DispatchResult {
		let cost_sum = Self::calculate_coupons_cost(a_coupon, b_coupon, c_coupon);
		ensure!(
			USDStore::<T>::get(who) >= cost_sum,
			Error::<T>::InsufficientUSDToRedeemCoupons
		);
		Ok(())
	}

	fn redeem_coupons(who: &Self::AccountId, a_coupon: u32, b_coupon: u32, c_coupon: u32) {
		let cost_sum = Self::calculate_coupons_cost(a_coupon, b_coupon, c_coupon);
		USDStore::<T>::mutate(who, |balance| *balance = balance.saturating_sub(cost_sum));
	}
}

impl<T: genesis_exchange::Config> genesis_exchange::Pallet<T> {
	fn machine_cost_by_id(cml_id: u64) -> Result<BalanceOf<T>, DispatchError> {
		let cml = T::CmlOperation::cml_by_id(&cml_id)?;
		let cost = match cml.cml_type() {
			CmlType::A => T::CmlAMiningMachineCost::get(),
			CmlType::B => T::CmlBMiningMachineCost::get(),
			CmlType::C => T::CmlCMiningMachineCost::get(),
		};
		Ok(cost)
	}

	fn calculate_coupons_cost(a_coupon: u32, b_coupon: u32, c_coupon: u32) -> BalanceOf<T> {
		let mut sum: BalanceOf<T> = Zero::zero();
		sum = sum.saturating_add(Self::coupon_type_cost(a_coupon, CmlType::A));
		sum = sum.saturating_add(Self::coupon_type_cost(b_coupon, CmlType::B));
		sum = sum.saturating_add(Self::coupon_type_cost(c_coupon, CmlType::C));
		sum
	}

	fn coupon_type_cost(amount: u32, cml_type: CmlType) -> BalanceOf<T> {
		let cost = match cml_type {
			CmlType::A => T::CmlARedeemCouponCost::get(),
			CmlType::B => T::CmlBRedeemCouponCost::get(),
			CmlType::C => T::CmlCRedeemCouponCost::get(),
		};
		cost.saturating_mul(amount.into())
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::*;
	use frame_support::{assert_noop, assert_ok};
	use pallet_cml::{CmlId, CmlType, DefrostScheduleType, Seed, CML};

	#[test]
	fn redeem_coupons_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(
				USDStore::<Test>::get(&COMPETITION_USERS1),
				COMPETITION_USER_USD_AMOUNT
			);
			assert_ok!(GenesisExchange::check_redeem_coupons(
				&COMPETITION_USERS1,
				1,
				2,
				4
			));
			GenesisExchange::redeem_coupons(&COMPETITION_USERS1, 1, 2, 4);
			assert_eq!(
				USDStore::<Test>::get(&COMPETITION_USERS1),
				COMPETITION_USER_USD_AMOUNT - 6000
			);

			assert_eq!(
				USDStore::<Test>::get(&COMPETITION_USERS2),
				COMPETITION_USER_USD_AMOUNT
			);
			assert_ok!(GenesisExchange::check_redeem_coupons(
				&COMPETITION_USERS2,
				1,
				2,
				4
			));
			GenesisExchange::redeem_coupons(&COMPETITION_USERS2, 1, 2, 4);
			assert_eq!(
				USDStore::<Test>::get(&COMPETITION_USERS2),
				COMPETITION_USER_USD_AMOUNT - 6000
			);

			USDStore::<Test>::insert(COMPETITION_USERS3, 0);
			assert_noop!(
				GenesisExchange::check_redeem_coupons(&COMPETITION_USERS3, 1, 1, 1),
				Error::<Test>::InsufficientUSDToRedeemCoupons
			);
			GenesisExchange::redeem_coupons(&COMPETITION_USERS3, 1, 1, 1);
			assert_eq!(USDStore::<Test>::get(&COMPETITION_USERS3), 0);
		})
	}

	#[test]
	fn single_coupon_cost_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(
				GenesisExchange::coupon_type_cost(1, CmlType::A),
				CML_A_REDEEM_COUPON_COST
			);

			assert_eq!(
				GenesisExchange::coupon_type_cost(3, CmlType::B),
				CML_B_REDEEM_COUPON_COST * 3
			);

			assert_eq!(
				GenesisExchange::coupon_type_cost(5, CmlType::C),
				CML_C_REDEEM_COUPON_COST * 5
			);
		})
	}

	#[test]
	fn buy_mining_machine_works() {
		new_test_ext().execute_with(|| {
			let user4 = 4;
			let cml_id1: CmlId = 11;
			let cml_id2: CmlId = 22;
			let cml_id3: CmlId = 33;
			Cml::add_cml(
				&user4,
				CML::from_genesis_seed(seed_from_type(cml_id1, CmlType::A)),
			);
			Cml::add_cml(
				&user4,
				CML::from_genesis_seed(seed_from_type(cml_id2, CmlType::B)),
			);
			Cml::add_cml(
				&user4,
				CML::from_genesis_seed(seed_from_type(cml_id3, CmlType::C)),
			);

			// let user4 has sufficient amount to buy a type C machine
			USDStore::<Test>::insert(user4, CML_C_MINING_MACHINE_COST);

			// user4 should fail if buy type A
			assert_noop!(
				GenesisExchange::check_buying_mining_machine(&user4, cml_id1),
				Error::<Test>::InsufficientUSDToPayMiningMachineCost
			);
			GenesisExchange::buy_mining_machine(&user4, cml_id1);
			assert_eq!(USDStore::<Test>::get(user4), CML_C_MINING_MACHINE_COST);

			// user4 should fail if buy type B
			assert_noop!(
				GenesisExchange::check_buying_mining_machine(&user4, cml_id2),
				Error::<Test>::InsufficientUSDToPayMiningMachineCost
			);
			GenesisExchange::buy_mining_machine(&user4, cml_id2);
			assert_eq!(USDStore::<Test>::get(user4), CML_C_MINING_MACHINE_COST);

			// user4 should success if buy type C
			assert_ok!(GenesisExchange::check_buying_mining_machine(
				&user4, cml_id3
			));
			GenesisExchange::buy_mining_machine(&user4, cml_id3);
			assert_eq!(USDStore::<Test>::get(user4), 0);

			// user4 should success if buy type A and B
			USDStore::<Test>::insert(user4, CML_A_MINING_MACHINE_COST + CML_B_MINING_MACHINE_COST);
			GenesisExchange::buy_mining_machine(&user4, cml_id1);
			GenesisExchange::buy_mining_machine(&user4, cml_id2);
			assert_eq!(USDStore::<Test>::get(user4), 0);
		})
	}

	fn seed_from_type(id: CmlId, cml_type: CmlType) -> Seed {
		Seed {
			id,
			cml_type,
			defrost_schedule: Some(DefrostScheduleType::Team),
			defrost_time: Some(0),
			lifespan: 100,
			performance: 0,
		}
	}
}
