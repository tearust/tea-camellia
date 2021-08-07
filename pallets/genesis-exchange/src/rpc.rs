use super::*;

impl<T: genesis_exchange::Config> genesis_exchange::Pallet<T> {
	/// current 1TEA equals how many USD amount.
	pub fn current_exchange_rate() -> BalanceOf<T> {
		let dollar = u128_to_balance::<T>(10_000_000_000 * 100);

		let exchange_remains_usd = USDStore::<T>::get(OperationAccount::<T>::get());
		let exchange_remains_tea =
			T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());
		Self::delta_deposit_amount(&dollar, &exchange_remains_tea, &exchange_remains_usd)
	}

	pub fn estimate_amount(withdraw_amount: BalanceOf<T>, buy_tea: bool) -> BalanceOf<T> {
		let exchange_remains_usd = USDStore::<T>::get(OperationAccount::<T>::get());
		let exchange_remains_tea =
			T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());

		match buy_tea {
			true => Self::delta_deposit_amount(
				&withdraw_amount,
				&exchange_remains_tea,
				&exchange_remains_usd,
			),
			false => Self::delta_deposit_amount(
				&withdraw_amount,
				&exchange_remains_usd,
				&exchange_remains_tea,
			),
		}
	}

	pub fn user_asset_list() -> Vec<(T::AccountId, BalanceOf<T>)> {
		let mut asset_usd_map = BTreeMap::new();
		Self::collect_cml_assets(&mut asset_usd_map);
		Self::collect_tea_assets(&mut asset_usd_map);
		Self::collect_usd_assets(&mut asset_usd_map);

		let mut total_assets: Vec<(T::AccountId, BalanceOf<T>)> = asset_usd_map
			.iter()
			.filter(|(user, _)| CompetitionUsers::<T>::contains_key(user))
			.map(|(user, reward)| (user.clone(), *reward))
			.collect();
		total_assets.sort_by(|(_, a), (_, b)| a.cmp(b));
		total_assets.reverse();
		total_assets
	}

	fn collect_usd_assets(asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>) {
		USDStore::<T>::iter()
			.for_each(|(user, amount)| Self::new_or_add_assets(&user, amount, asset_usd_map))
	}

	fn collect_tea_assets(asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>) {
		CompetitionUsers::<T>::iter().for_each(|(user, _)| {
			let tea_amount = T::CurrencyOperations::free_balance(&user);
			Self::new_or_add_assets(
				&user,
				tea_amount * Self::current_exchange_rate(),
				asset_usd_map,
			)
		});
	}

	fn collect_cml_assets(asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>) {
		// calculate reward statement of current block, we assume each mining cml will get the
		// mining change equally, and each mining task point are same.
		let cml_reward_statements = T::CmlOperation::estimate_reward_statements(
			|| T::CmlOperation::current_mining_cmls().iter().count() as u32,
			|_cml_id| 1u32,
		);
		for (user, _, single_block_reward) in cml_reward_statements {
			let reward_in_tea = Self::estimate_cml_asset_value(single_block_reward);
			let reward_in_usd = reward_in_tea * Self::current_exchange_rate();

			Self::new_or_add_assets(&user, reward_in_usd, asset_usd_map);
		}
	}

	fn new_or_add_assets(
		user: &T::AccountId,
		amount: BalanceOf<T>,
		asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>,
	) {
		if let Some(old) = asset_usd_map.remove(user) {
			asset_usd_map.insert(user.clone(), old + amount);
		} else {
			asset_usd_map.insert(user.clone(), amount);
		}
	}

	fn estimate_cml_asset_value(single_block_reward: BalanceOf<T>) -> BalanceOf<T> {
		Self::reward_of_one_day(single_block_reward) * T::PER::get()
	}

	fn reward_of_one_day(single_block_reward: BalanceOf<T>) -> BalanceOf<T> {
		// average block timespan is 6 seconds
		single_block_reward * (10u32 * 60u32 * 24u32).into()
	}
}

fn u128_to_balance<T: Config>(amount: u128) -> BalanceOf<T> {
	amount.try_into().map_err(|_| "").unwrap()
}
