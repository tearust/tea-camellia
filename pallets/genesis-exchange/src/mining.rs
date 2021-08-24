use super::*;
use pallet_cml::{CmlType, SeedProperties};

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
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::*;
	use frame_support::{assert_noop, assert_ok};
	use pallet_cml::{CmlId, CmlType, DefrostScheduleType, Seed, CML};

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
