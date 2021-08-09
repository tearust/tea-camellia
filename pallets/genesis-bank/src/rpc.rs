use super::*;

impl<T: genesis_bank::Config> genesis_bank::Pallet<T> {
	pub fn cml_calculate_loan_amount(cml_id: u64, block_height: T::BlockNumber) -> BalanceOf<T> {
		let unique_id = AssetUniqueId {
			asset_type: AssetType::CML,
			inner_id: from_cml_id(cml_id),
		};
		Self::cml_need_to_pay(&unique_id, &block_height)
	}

	pub fn user_collateral_list(who: &T::AccountId) -> Vec<(u64, T::BlockNumber)> {
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
