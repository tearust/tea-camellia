use super::*;

impl<T: genesis_bank::Config> genesis_bank::Pallet<T> {
	pub fn cml_calculate_loan_amount(cml_id: u64, block_height: T::BlockNumber) -> BalanceOf<T> {
		let unique_id = AssetUniqueId {
			asset_type: AssetType::CML,
			inner_id: from_cml_id(cml_id),
		};
		Self::cml_need_to_pay(&unique_id, &block_height)
	}

	pub fn user_cml_lien_list(who: &T::AccountId) -> Vec<u64> {
		UserCollateralStore::<T>::iter_prefix(who)
			.map(|(id, _)| to_cml_id(&id.inner_id).unwrap_or(u64::MAX))
			.collect()
	}

	pub fn bank_owned_cmls() -> Vec<u64> {
		T::CmlOperation::user_owned_cmls(&OperationAccount::<T>::get())
			.iter()
			.filter(|id| {
				let unique_id = AssetUniqueId {
					asset_type: AssetType::CML,
					inner_id: from_cml_id(**id),
				};
				!CollateralStore::<T>::contains_key(&unique_id)
			})
			.map(|id| *id)
			.collect()
	}
}
