use super::*;

impl<T: genesis_bank::Config> genesis_bank::Pallet<T> {
	pub fn cml_calculate_loan_amount(cml_id: u64, block_height: T::BlockNumber) -> BalanceOf<T> {
		Self::calculate_loan_amount(cml_id, block_height)
	}

	pub fn user_collateral_list(who: &T::AccountId) -> Vec<(u64, T::BlockNumber)> {
		Self::user_collaterals(who)
	}
}

impl<T: genesis_bank::Config> GenesisBankOperation for genesis_bank::Pallet<T> {
	type AccountId = T::AccountId;
	type BlockNumber = T::BlockNumber;
	type Balance = BalanceOf<T>;

	fn is_cml_in_loan(cml_id: CmlId) -> bool {
		CollateralStore::<T>::contains_key(AssetUniqueId {
			asset_type: AssetType::CML,
			inner_id: from_cml_id(cml_id),
		})
	}

	fn calculate_loan_amount(cml_id: u64, block_height: Self::BlockNumber) -> Self::Balance {
		let unique_id = AssetUniqueId {
			asset_type: AssetType::CML,
			inner_id: from_cml_id(cml_id),
		};
		Self::cml_need_to_pay(&unique_id, &block_height)
	}

	fn user_collaterals(who: &Self::AccountId) -> Vec<(CmlId, Self::BlockNumber)> {
		UserCollateralStore::<T>::iter_prefix(who)
			.map(|(id, _)| {
				(
					to_cml_id(&id.inner_id).unwrap_or(u64::MAX),
					CollateralStore::<T>::get(&id).start_at + T::LoanTermDuration::get(),
				)
			})
			.collect()
	}
}
