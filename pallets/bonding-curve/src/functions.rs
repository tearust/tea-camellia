use super::*;
use pallet_cml::TreeProperties;

pub(crate) const CALCULATION_PRECISION: u32 = 100000000;

impl<T: bonding_curve::Config> BondingCurveOperation for bonding_curve::Pallet<T> {
	type AccountId = T::AccountId;
	type Balance = BalanceOf<T>;

	fn list_tapp_ids() -> Vec<u64> {
		TAppBondingCurve::<T>::iter().map(|(id, _)| id).collect()
	}

	fn estimate_hosting_income_statements(
		tapp_id: u64,
	) -> Vec<(Self::AccountId, CmlId, Self::Balance)> {
		let tapp = TAppBondingCurve::<T>::get(tapp_id);
		match tapp.billing_mode {
			BillingMode::FixedHostingFee(reward_per_performance) => {
				let host_count = TAppCurrentHosts::<T>::iter_prefix(tapp_id).count() as u32;
				let tea_amount = tapp.current_cost.saturating_add(
					reward_per_performance
						.saturating_mul(tapp.host_performance().into())
						.saturating_mul(host_count.into()),
				);

				if let Ok((_, distribute_tea_amount, _)) =
					Self::calculate_given_received_tea_how_much_seller_give_away(
						tapp_id, tea_amount,
					) {
					return match Self::distribute_to_miners(tapp_id, distribute_tea_amount, false) {
						Ok(statements) => statements,
						Err(_) => vec![],
					};
				}
			}
			_ => {}
		}

		vec![]
	}

	fn current_price(tapp_id: u64) -> (Self::Balance, Self::Balance) {
		Self::query_price(tapp_id)
	}

	fn tapp_user_balances(who: &Self::AccountId) -> Vec<(u64, Self::Balance)> {
		AccountTable::<T>::iter_prefix(who).collect()
	}
}

impl<T: bonding_curve::Config> bonding_curve::Pallet<T> {
	pub(crate) fn need_arrange_host(height: T::BlockNumber) -> bool {
		// offset with `InterestPeriodLength` - 3 to void overlapping with staking period
		height % T::HostArrangeDuration::get() == T::HostArrangeDuration::get() - 3u32.into()
	}

	pub(crate) fn need_collect_host_cost(height: T::BlockNumber) -> bool {
		height % T::HostCostCollectionDuration::get()
			== T::HostCostCollectionDuration::get() - 4u32.into()
	}

	pub fn next_id() -> TAppId {
		LastTAppId::<T>::mutate(|id| {
			if *id < u64::MAX {
				*id += 1;
			} else {
				*id = 1;
			}

			*id
		})
	}

	pub fn allocate_buy_tea_amount(
		who: &T::AccountId,
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let deposit_tea_amount = Self::calculate_buy_amount(Some(tapp_id), tapp_amount)?;
		let reserved_tea_amount = Self::calculate_raise_reserve_amount(tapp_id, tapp_amount)?;
		ensure!(
			deposit_tea_amount >= reserved_tea_amount,
			Error::<T>::SubtractionOverflow
		);

		T::CurrencyOperations::transfer(
			who,
			&ReservedBalanceAccount::<T>::get(),
			reserved_tea_amount,
			ExistenceRequirement::AllowDeath,
		)?;
		T::CurrencyOperations::transfer(
			who,
			&TAppBondingCurve::<T>::get(tapp_id).owner,
			deposit_tea_amount.saturating_sub(reserved_tea_amount),
			ExistenceRequirement::AllowDeath,
		)?;

		Ok(deposit_tea_amount)
	}

	pub fn buy_token_inner(
		who: &T::AccountId,
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		match Self::allocate_buy_tea_amount(who, tapp_id, tapp_amount) {
			Ok(deposit_tea_amount) => {
				AccountTable::<T>::mutate(who, tapp_id, |amount| {
					*amount = amount.saturating_add(tapp_amount);
				});
				TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
					*amount = amount.saturating_add(tapp_amount);
				});

				deposit_tea_amount
			}
			Err(e) => {
				// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
				log::error!("buy token inner error: {:?}", e);
				Zero::zero()
			}
		}
	}

	pub fn sell_token_inner(
		who: &T::AccountId,
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		if let Err(e) = AccountTable::<T>::mutate(who, tapp_id, |amount| {
			match amount.checked_sub(&tapp_amount) {
				Some(a) => {
					*amount = a;
					Ok(())
				}
				None => Err("account tapp token is not enough"),
			}
		}) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			log::error!("{}", e);
			return Zero::zero();
		}

		match Self::calculate_sell_amount(tapp_id, tapp_amount) {
			Ok(deposit_tea_amount) => {
				if let Err(e) = T::CurrencyOperations::transfer(
					&ReservedBalanceAccount::<T>::get(),
					who,
					deposit_tea_amount,
					ExistenceRequirement::AllowDeath,
				) {
					// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
					log::error!("transfer free balance failed: {:?}", e);
					return Zero::zero();
				}

				TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
					*amount = amount.saturating_sub(tapp_amount);
				});
				Self::try_clean_tapp_related(Some(who.clone()), tapp_id);

				deposit_tea_amount
			}
			Err(e) => {
				// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
				log::error!("calculating sell amount failed: {:?}", e);
				return Zero::zero();
			}
		}
	}

	pub fn try_clean_tapp_related(who: Option<T::AccountId>, tapp_id: TAppId) {
		match who {
			Some(who) => {
				if AccountTable::<T>::get(&who, tapp_id).is_zero() {
					AccountTable::<T>::remove(&who, tapp_id);
				}
			}
			None => AccountTable::<T>::iter()
				.filter(|(_, id, balance)| *id == tapp_id && balance.is_zero())
				.for_each(|(acc, id, _)| AccountTable::<T>::remove(acc, id)),
		}

		if TotalSupplyTable::<T>::get(tapp_id).is_zero() {
			TotalSupplyTable::<T>::remove(tapp_id);
			let item = TAppBondingCurve::<T>::take(tapp_id);
			TAppNames::<T>::remove(item.name);
			TAppTickers::<T>::remove(item.ticker);
			TAppCurrentHosts::<T>::iter_prefix(tapp_id)
				.for_each(|(cml_id, _)| Self::unhost_tapp(tapp_id, cml_id));
		}
	}

	pub(crate) fn distribute_to_investors(tapp_id: TAppId, distributing_amount: BalanceOf<T>) {
		// todo replace total amount with total supply later if calculation result is correct
		let (investors, mut total_amount) = Self::tapp_investors(tapp_id);

		match TAppBondingCurve::<T>::get(tapp_id).billing_mode {
			BillingMode::FixedHostingToken(_) => {
				TAppReservedBalance::<T>::iter_prefix(tapp_id).for_each(|(_, amount)| {
					total_amount = total_amount.saturating_add(amount);
				});

				TAppReservedBalance::<T>::iter_prefix(tapp_id).for_each(
					|(account, reserved_amount)| {
						AccountTable::<T>::mutate(account, tapp_id, |user_amount| {
							*user_amount = user_amount.saturating_add(
								distributing_amount * reserved_amount / total_amount,
							);
						});
					},
				);
			}
			_ => {}
		}

		investors.iter().for_each(|account| {
			AccountTable::<T>::mutate(account, tapp_id, |user_amount| {
				*user_amount =
					user_amount.saturating_add(distributing_amount * (*user_amount) / total_amount);
			});
		});

		TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
			*amount = amount.saturating_add(distributing_amount);
		});
	}

	pub(crate) fn collect_with_investors(tapp_id: TAppId, collecting_amount: BalanceOf<T>) {
		// todo replace total amount with total supply later if calculation result is correct
		let (investors, total_amount) = Self::tapp_investors(tapp_id);
		if !approximately_equals::<T>(
			total_amount,
			TotalSupplyTable::<T>::get(tapp_id),
			CALCULATION_PRECISION.into(),
		) {
			log::error!(
				"collecting calculate total amount error: calculated result is: {:?}, \
				total supply is {:?}",
				total_amount,
				TotalSupplyTable::<T>::get(tapp_id)
			);
		}

		investors.iter().for_each(|account| {
			AccountTable::<T>::mutate(account, tapp_id, |user_amount| {
				*user_amount =
					user_amount.saturating_sub(collecting_amount * (*user_amount) / total_amount);
			});
		});

		TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
			*amount = amount.saturating_sub(collecting_amount);
		});
	}

	pub(crate) fn tapp_investors(tapp_id: TAppId) -> (BTreeSet<T::AccountId>, BalanceOf<T>) {
		let mut investors = BTreeSet::new();
		let mut total_amount: BalanceOf<T> = Zero::zero();
		for (account, id, amount) in AccountTable::<T>::iter() {
			if id != tapp_id {
				continue;
			}
			total_amount = total_amount.saturating_add(amount);
			investors.insert(account);
		}

		(investors, total_amount)
	}

	pub(crate) fn calculate_buy_amount(
		tapp_id: Option<TAppId>,
		tapp_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		match tapp_id {
			Some(tapp_id) => {
				let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
				let total_supply = TotalSupplyTable::<T>::get(tapp_id);
				Self::calculate_increase_amount_from_raise_curve_total_supply(
					tapp_item.buy_curve,
					total_supply,
					tapp_amount,
				)
			}
			None => {
				// by default total supply is zero, buy curve is UnsignedSquareRoot_10
				Self::calculate_increase_amount_from_raise_curve_total_supply(
					CurveType::UnsignedSquareRoot_10,
					Zero::zero(),
					tapp_amount,
				)
			}
		}
	}

	pub(crate) fn calculate_raise_reserve_amount(
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		Self::calculate_increase_amount_from_raise_curve_total_supply(
			tapp_item.sell_curve,
			total_supply,
			tapp_amount,
		)
	}

	pub(crate) fn calculate_increase_amount_from_raise_curve_total_supply(
		curve_type: CurveType,
		total_supply: BalanceOf<T>,
		tapp_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let current_pool_balance = match curve_type {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply)
			}
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
		};

		let after_buy_pool_balance = match curve_type {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(
				total_supply
					.checked_add(&tapp_amount)
					.ok_or(Error::<T>::AddOverflow)?,
			),
			CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::pool_balance(
				total_supply
					.checked_add(&tapp_amount)
					.ok_or(Error::<T>::AddOverflow)?,
			),
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(
				total_supply
					.checked_add(&tapp_amount)
					.ok_or(Error::<T>::AddOverflow)?,
			),
		};
		Ok(after_buy_pool_balance
			.checked_sub(&current_pool_balance)
			.ok_or(Error::<T>::SubtractionOverflow)?)
	}

	pub(crate) fn calculate_given_increase_tea_how_much_token_mint(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let current_buy_area_tea_amount = match tapp_item.buy_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply)
			}
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
		};
		let after_increase_tea_amount = current_buy_area_tea_amount
			.checked_add(&tea_amount)
			.ok_or(Error::<T>::AddOverflow)?;
		let total_supply_after_increase = match tapp_item.buy_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance_reverse(
				after_increase_tea_amount,
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::pool_balance_reverse(
				after_increase_tea_amount,
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance_reverse(
				after_increase_tea_amount,
				T::PoolBalanceReversePrecision::get(),
			),
		};
		Ok(total_supply_after_increase
			.checked_sub(&total_supply)
			.ok_or(Error::<T>::SubtractionOverflow)?)
	}

	// pub(crate) fn calculate_decrease_amount_from_reduce_curve_total_supply(
	// 	curve_type: CurveType,
	// 	total_supply: BalanceOf<T>,
	// 	tapp_amount: BalanceOf<T>,
	// ) -> Result<BalanceOf<T>, DispatchError> {
	// 	let current_pool_balance = match curve_type {
	// 		CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
	// 		CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::pool_balance(total_supply),
	// 		CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
	// 	};

	// 	let after_sell_pool_balance = match curve_type {
	// 		CurveType::UnsignedLinear => T::LinearCurve::pool_balance(
	// 			total_supply
	// 				.checked_sub(&tapp_amount)
	// 				.ok_or(Error::<T>::AddOverflow)?,
	// 		),
	// 		CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::pool_balance(
	// 			total_supply
	// 				.checked_sub(&tapp_amount)
	// 				.ok_or(Error::<T>::AddOverflow)?,
	// 		),
	// 		CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(
	// 			total_supply
	// 				.checked_sub(&tapp_amount)
	// 				.ok_or(Error::<T>::AddOverflow)?,
	// 		),
	// 	};
	// 	Ok(
	// 		current_pool_balance
	// 			.checked_sub(&after_sell_pool_balance)
	// 			.ok_or(Error::<T>::SubtractionOverflow)?,
	// 	)
	// }

	/// If user want to sell tapp_amount of tapp_id token, how many T token seller receive after sale
	pub(crate) fn calculate_sell_amount(
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		ensure!(
			tapp_amount <= total_supply,
			Error::<T>::InsufficientTotalSupply
		);

		let current_pool_balance = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply)
			}
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
		};
		let after_sell_pool_balance = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => {
				T::LinearCurve::pool_balance(total_supply.saturating_sub(tapp_amount))
			}
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply.saturating_sub(tapp_amount))
			}
			CurveType::UnsignedSquareRoot_7 => {
				T::UnsignedSquareRoot_7::pool_balance(total_supply.saturating_sub(tapp_amount))
			}
		};
		Ok(current_pool_balance
			.checked_sub(&after_sell_pool_balance)
			.ok_or(Error::<T>::SubtractionOverflow)?)
	}

	/// calcualte given seller receive tea_amount of TEA, how much of tapp token this seller will give away
	///
	/// returns:
	/// - really given tapp amount
	/// - really payed tea amount
	/// - is reserved tea zero
	pub(crate) fn calculate_given_received_tea_how_much_seller_give_away(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>, bool), DispatchError> {
		let mut is_reserved_tea_zero = false;

		let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let current_reserve_pool_tea = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply)
			}
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
		};

		let pay_amount = if current_reserve_pool_tea < tea_amount {
			is_reserved_tea_zero = true;
			current_reserve_pool_tea
		} else {
			tea_amount
		};

		let total_supply_after_sell_tapp_token = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance_reverse(
				current_reserve_pool_tea.saturating_sub(pay_amount),
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::pool_balance_reverse(
				current_reserve_pool_tea.saturating_sub(pay_amount),
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance_reverse(
				current_reserve_pool_tea.saturating_sub(pay_amount),
				T::PoolBalanceReversePrecision::get(),
			),
		};
		Ok((
			total_supply
				.checked_sub(&total_supply_after_sell_tapp_token)
				.ok_or(Error::<T>::SubtractionOverflow)?,
			pay_amount,
			is_reserved_tea_zero,
		))
	}

	pub(crate) fn check_tapp_fields_length(
		tapp_name: &Vec<u8>,
		ticker: &Vec<u8>,
		detail: &Vec<u8>,
		link: &Vec<u8>,
	) -> DispatchResult {
		ensure!(
			tapp_name.len() <= T::TAppNameMaxLength::get() as usize,
			Error::<T>::TAppNameIsTooLong
		);
		ensure!(
			ticker.len() <= T::TAppTickerMaxLength::get() as usize,
			Error::<T>::TAppTickerIsTooLong
		);
		ensure!(
			ticker.len() >= T::TAppTickerMinLength::get() as usize,
			Error::<T>::TAppTickerIsTooShort
		);
		ensure!(
			detail.len() <= T::TAppDetailMaxLength::get() as usize,
			Error::<T>::TAppDetailIsTooLong
		);
		ensure!(
			link.len() <= T::TAppLinkMaxLength::get() as usize,
			Error::<T>::TAppLinkIsTooLong
		);
		Ok(())
	}

	pub(crate) fn check_host_creating(
		max_allowed_hosts: u32,
		fixed_token_mode: bool,
		reward_per_performance: &Option<BalanceOf<T>>,
		stake_token_amount: &Option<BalanceOf<T>>,
	) -> DispatchResult {
		ensure!(
			max_allowed_hosts >= T::MinTappHostsCount::get(),
			Error::<T>::MaxAllowedHostShouldLargerEqualThanMinAllowedHosts,
		);

		if fixed_token_mode {
			ensure!(
				stake_token_amount.is_some(),
				Error::<T>::StakeTokenIsNoneInFixedTokenMode
			);
			ensure!(
				!stake_token_amount.unwrap().is_zero(),
				Error::<T>::StakeTokenShouldNotBeZero
			);
		} else {
			ensure!(
				reward_per_performance.is_some(),
				Error::<T>::RewardPerPerformanceIsNoneInFixedFeeMode
			);
			ensure!(
				!reward_per_performance.unwrap().is_zero(),
				Error::<T>::RewardPerPerformanceShouldNotBeZero
			);
		}

		Ok(())
	}

	pub(crate) fn unhost_tapp(tapp_id: TAppId, cml_id: CmlId) {
		TAppCurrentHosts::<T>::remove(tapp_id, cml_id);

		match TAppBondingCurve::<T>::get(tapp_id).billing_mode {
			BillingMode::FixedHostingToken(_) => {
				if let Ok(cml) = T::CmlOperation::cml_by_id(&cml_id) {
					if let Some(owner) = cml.owner() {
						TAppReservedBalance::<T>::remove(tapp_id, owner);
					}
				}
			}
			_ => {}
		}

		CmlHostingTApps::<T>::mutate(cml_id, |array| {
			if let Some(index) = array.iter().position(|x| *x == tapp_id) {
				array.remove(index);
			}
		});

		Self::try_deactive_tapp(tapp_id);
	}

	pub(crate) fn try_active_tapp(tapp_id: TAppId) {
		if TAppBondingCurve::<T>::get(tapp_id).status == TAppStatus::Pending
			&& TAppCurrentHosts::<T>::iter_prefix(tapp_id).count()
				>= T::MinTappHostsCount::get() as usize
		{
			TAppBondingCurve::<T>::mutate(tapp_id, |tapp| tapp.status = TAppStatus::Active);
		}
	}

	pub(crate) fn try_deactive_tapp(tapp_id: TAppId) {
		if TAppBondingCurve::<T>::get(tapp_id).status == TAppStatus::Active
			&& TAppCurrentHosts::<T>::iter_prefix(tapp_id).count()
				< T::MinTappHostsCount::get() as usize
		{
			TAppBondingCurve::<T>::mutate(tapp_id, |tapp| tapp.status = TAppStatus::Pending);
		}
	}

	pub(crate) fn unhost_last_tapp(cml_id: CmlId) -> Option<TAppId> {
		if let Some(last_tapp) = CmlHostingTApps::<T>::get(cml_id).last() {
			Self::unhost_tapp(*last_tapp, cml_id);
			return Some(*last_tapp);
		}
		None
	}

	pub(crate) fn arrange_host() {
		let current_block = frame_system::Pallet::<T>::block_number();
		let mining_cmls = T::CmlOperation::current_mining_cmls();

		let mut unhosted_list = Vec::new();
		mining_cmls.iter().for_each(|cml_id| {
			let (current_performance, _) =
				T::CmlOperation::miner_performance(*cml_id, &current_block);
			while Self::cml_total_used_performance(*cml_id) > current_performance.unwrap_or(0) {
				if let Some(tapp_id) = Self::unhost_last_tapp(*cml_id) {
					unhosted_list.push((tapp_id, *cml_id));
				}
			}
		});

		Self::deposit_event(Event::TAppsUnhosted(unhosted_list));
	}

	pub(crate) fn cml_total_used_performance(cml_id: CmlId) -> Performance {
		let mut total: Performance = Zero::zero();
		for tapp_id in CmlHostingTApps::<T>::get(cml_id).iter() {
			total = total.saturating_add(TAppBondingCurve::<T>::get(tapp_id).host_performance());
		}
		total
	}

	pub(crate) fn collect_host_cost() {
		TAppBondingCurve::<T>::iter()
			.filter(|(_, tapp)| match tapp.billing_mode {
				BillingMode::FixedHostingFee(_) => true,
				_ => false,
			})
			.for_each(|(id, tapp)| match tapp.billing_mode {
				BillingMode::FixedHostingFee(reward_per_performance) => {
					Self::accumulate_tapp_cost(id, reward_per_performance);
					Self::expense_inner(id)
				}
				_ => {}
			});
	}

	pub(crate) fn accumulate_tapp_cost(tapp_id: TAppId, reward_per_performance: BalanceOf<T>) {
		TAppBondingCurve::<T>::mutate(tapp_id, |tapp| {
			let host_count = TAppCurrentHosts::<T>::iter_prefix(tapp_id).count() as u32;
			tapp.current_cost = tapp.current_cost.saturating_add(
				reward_per_performance
					.saturating_mul(tapp.host_performance().into())
					.saturating_mul(host_count.into()),
			);
		});
	}

	pub(crate) fn distribute_to_miners(
		tapp_id: TAppId,
		total_amount: BalanceOf<T>,
		do_transfer: bool,
	) -> Result<Vec<(T::AccountId, CmlId, BalanceOf<T>)>, DispatchError> {
		let host_count = TAppCurrentHosts::<T>::iter_prefix(tapp_id).count() as u32;
		ensure!(
			!host_count.is_zero(),
			Error::<T>::NoHostingToDistributeMiner
		);

		let each_amount = total_amount / host_count.into();

		let mut tapp_statements = Vec::new();
		for (cml_id, _) in TAppCurrentHosts::<T>::iter_prefix(tapp_id) {
			let staking_snapshots = T::CmlOperation::cml_staking_snapshots(cml_id);
			let mut statements = T::CmlOperation::single_cml_staking_reward_statements(
				cml_id,
				&staking_snapshots,
				each_amount,
			);

			if do_transfer {
				for (account, _, amount) in statements.iter() {
					T::CurrencyOperations::transfer(
						&ReservedBalanceAccount::<T>::get(),
						account,
						amount.clone(),
						ExistenceRequirement::AllowDeath,
					)?;
				}
			}

			tapp_statements.append(&mut statements);
		}
		Ok(tapp_statements)
	}

	pub fn expense_inner(tapp_id: TAppId) {
		let tapp = TAppBondingCurve::<T>::get(tapp_id);
		if tapp.current_cost.is_zero() {
			return;
		}

		match Self::calculate_given_received_tea_how_much_seller_give_away(
			tapp_id,
			tapp.current_cost,
		) {
			Ok((withdraw_tapp_amount, distribute_tea_amount, is_reserved_tea_zero)) => {
				match Self::distribute_to_miners(tapp_id, distribute_tea_amount, true) {
					Ok(tapp_statements) => {
						Self::collect_with_investors(tapp_id, withdraw_tapp_amount);
						TAppBondingCurve::<T>::mutate(tapp_id, |tapp_item| {
							tapp_item.current_cost = Zero::zero();
						});

						let (buy_price, sell_price) = Self::query_price(tapp_id);
						Self::deposit_event(Event::TAppExpense(
							tapp_id,
							tapp_statements,
							buy_price,
							sell_price,
							TotalSupplyTable::<T>::get(tapp_id),
						));
					}
					Err(e) => {
						// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
						log::error!("transfer free balance failed: {:?}", e);
					}
				}

				if is_reserved_tea_zero {
					Self::bankrupt_tapp(tapp_id);
				}
			}
			Err(e) => {
				// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
				log::error!("calculation failed: {:?}", e);
			}
		}
	}

	pub(crate) fn bankrupt_tapp(tapp_id: TAppId) {
		Self::try_clean_tapp_related(None, tapp_id);
		Self::deposit_event(Event::TAppBankrupted(tapp_id));
	}
}

pub fn approximately_equals<T>(a: BalanceOf<T>, b: BalanceOf<T>, precision: BalanceOf<T>) -> bool
where
	T: bonding_curve::Config,
{
	let abs = match a >= b {
		true => a.saturating_sub(b),
		false => b.saturating_sub(a),
	};
	abs <= precision
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::tests::{create_default_tapp, seed_from_lifespan};
	use crate::*;
	use bonding_curve_impl::approximately_equals;
	use frame_support::assert_ok;
	use pallet_cml::{CmlStore, UserCmlStore, CML};

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

	#[test]
	fn allocate_buy_tea_amount_works() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let user3 = 3;
			let tapp_id = 1;
			<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
			<Test as Config>::Currency::make_free_balance_be(&user3, DOLLARS);
			TAppBondingCurve::<Test>::insert(
				tapp_id,
				TAppItem {
					id: tapp_id,
					owner: user2,
					buy_curve: CurveType::UnsignedSquareRoot_10,
					sell_curve: CurveType::UnsignedSquareRoot_7,
					..Default::default()
				},
			);
			assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), 0);

			let deposit_amount = BondingCurve::allocate_buy_tea_amount(&user1, 1, 1_000_000);
			assert_eq!(deposit_amount, Ok(666));
			assert_eq!(<Test as Config>::Currency::free_balance(&user2), 200);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user1),
				DOLLARS - 666
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&ReservedBalanceAccount::<Test>::get()),
				466
			);

			TotalSupplyTable::<Test>::insert(tapp_id, 1_000_000);
			let deposit_amount = BondingCurve::allocate_buy_tea_amount(&user3, 1, 9_000_000);
			assert_eq!(deposit_amount.unwrap(), 20414);
			assert_eq!(<Test as Config>::Currency::free_balance(&user2), 6324);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user3),
				999999979586
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&ReservedBalanceAccount::<Test>::get()),
				14756
			);
		})
	}

	#[test]
	fn calculate_given_increase_tea_how_much_token_mint_works() {
		new_test_ext().execute_with(|| {
			let tapp_id = 1;
			TotalSupplyTable::<Test>::insert(tapp_id, 0);
			TAppBondingCurve::<Test>::insert(
				tapp_id,
				TAppItem {
					id: tapp_id,
					buy_curve: CurveType::UnsignedSquareRoot_10,
					sell_curve: CurveType::UnsignedSquareRoot_7,
					..Default::default()
				},
			);
			let amount = BondingCurve::calculate_given_increase_tea_how_much_token_mint(
				tapp_id,
				666666666666,
			);
			assert!(approximately_equals(
				amount.unwrap(),
				1_000_000_000_000,
				1000
			));
			TotalSupplyTable::<Test>::insert(tapp_id, 1_000_000_000_000);
			let amount = BondingCurve::calculate_given_increase_tea_how_much_token_mint(
				tapp_id,
				666666666666,
			);
			// println!("amt {:?}", &amount);
			assert!(approximately_equals(amount.unwrap(), 587401114832, 100));
		})
	}
	#[test]
	fn calculate_sell_amount_works() {
		new_test_ext().execute_with(|| {
			let tapp_id = 1;
			TotalSupplyTable::<Test>::insert(tapp_id, 100_000_000);
			TAppBondingCurve::<Test>::insert(
				tapp_id,
				TAppItem {
					id: tapp_id,
					buy_curve: CurveType::UnsignedSquareRoot_10,
					sell_curve: CurveType::UnsignedSquareRoot_7,
					..Default::default()
				},
			);

			let amount = BondingCurve::calculate_sell_amount(tapp_id, 90_000_000);
			assert_eq!(amount.unwrap(), 451910);
		})
	}

	#[test]
	fn accumulate_tapp_cost_works() {
		new_test_ext().execute_with(|| {
			EnableUserCreateTApp::<Test>::set(true);
			pub const HOST_COST_COEFFICIENT: u128 = 10000;
			let miner = 2;
			let tapp_owner = 1;
			<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

			let cml_id = 11;
			let cml_id2 = 22;
			let performance = 1000u32;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 10000, 10000));
			let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 10000, 10000));
			UserCmlStore::<Test>::insert(miner, cml_id, ());
			UserCmlStore::<Test>::insert(miner, cml_id2, ());
			CmlStore::<Test>::insert(cml_id, cml);
			CmlStore::<Test>::insert(cml_id2, cml2);

			assert_ok!(Cml::start_mining(
				Origin::signed(miner),
				cml_id,
				[1u8; 32],
				b"miner_ip".to_vec()
			));
			assert_ok!(Cml::start_mining(
				Origin::signed(miner),
				cml_id2,
				[2u8; 32],
				b"miner_ip".to_vec()
			));

			assert_ok!(create_default_tapp(tapp_owner));

			let tapp_id = 1;
			assert_eq!(TAppBondingCurve::<Test>::get(tapp_id).current_cost, 0);

			BondingCurve::accumulate_tapp_cost(tapp_id, HOST_COST_COEFFICIENT);
			// Right now, there is zero host. the cost should be zero too
			assert_eq!(
				TAppBondingCurve::<Test>::get(tapp_id).current_cost,
				// HOST_COST_COEFFICIENT.saturating_mul(performance.into())
				0
			);

			// add one host, the cost should be 1000*HostCostCoefficient
			TAppBondingCurve::<Test>::mutate(tapp_id, |tapp_item| tapp_item.current_cost = 0);
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));
			BondingCurve::accumulate_tapp_cost(tapp_id, HOST_COST_COEFFICIENT);
			assert_eq!(
				TAppBondingCurve::<Test>::get(tapp_id).current_cost,
				HOST_COST_COEFFICIENT.saturating_mul(performance.into())
			);

			// Add second host, the cost should be 1000*HostCostCoefficient*2
			TAppBondingCurve::<Test>::mutate(tapp_id, |tapp_item| tapp_item.current_cost = 0);
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id2, tapp_id));
			BondingCurve::accumulate_tapp_cost(tapp_id, HOST_COST_COEFFICIENT);
			assert_eq!(
				TAppBondingCurve::<Test>::get(tapp_id).current_cost,
				HOST_COST_COEFFICIENT.saturating_mul((performance * 2).into())
			);

			frame_system::Pallet::<Test>::set_block_number(1001);
			// remove the first host, the cost should be 1000*HostCostCoefficient
			TAppBondingCurve::<Test>::mutate(tapp_id, |tapp_item| tapp_item.current_cost = 0);
			assert_ok!(BondingCurve::unhost(Origin::signed(miner), cml_id, tapp_id));
			BondingCurve::accumulate_tapp_cost(tapp_id, HOST_COST_COEFFICIENT);
			assert_eq!(
				TAppBondingCurve::<Test>::get(tapp_id).current_cost,
				HOST_COST_COEFFICIENT.saturating_mul(performance.into())
			);
		})
	}

	// #[test]
	// fn calculate_given_received_tea_how_much_seller_give_away_works() {
	// 	new_test_ext().execute_with(|| {
	// 		let tapp_id = 1;
	// 		TotalSupplyTable::<Test>::insert(tapp_id, 100_000_000_000_000);
	// 		TAppBondingCurve::<Test>::insert(
	// 			tapp_id,
	// 			TAppItem {
	// 				id: tapp_id,
	// 				buy_curve: CurveType::UnsignedSquareRoot_10,
	// 				sell_curve: CurveType::UnsignedSquareRoot_7,
	// 				..Default::default()
	// 			},
	// 		);
	// 		let token = BondingCurve::calculate_decrease_amount_from_reduce_curve_total_supply(
	// 			CurveType::UnsignedSquareRoot_7,
	// 			100_000_000_000_000,
	// 			1_000_000_000_000,
	// 		);
	// 		let amount = BondingCurve::calculate_given_received_tea_how_much_seller_give_away(
	// 			tapp_id,
	// 			token.unwrap(),
	// 		);
	// 		println!("amount {:?}", amount);
	// 		assert!(approximately_equals(amount.unwrap(), 1_000_000_000_000, 10));
	// 	})
	// }
}
