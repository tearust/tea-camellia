use super::*;
use pallet_cml::TreeProperties;

pub(crate) const CALCULATION_PRECISION: u32 = 100000000;

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
			&OperationAccount::<T>::get(),
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
					&OperationAccount::<T>::get(),
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
				Self::try_clean_tapp_related(who, tapp_id);

				deposit_tea_amount
			}
			Err(e) => {
				// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
				log::error!("calculating sell amount failed: {:?}", e);
				return Zero::zero();
			}
		}
	}

	pub fn try_clean_tapp_related(who: &T::AccountId, tapp_id: TAppId) {
		if AccountTable::<T>::get(who, tapp_id).is_zero() {
			AccountTable::<T>::remove(who, tapp_id);
		}
		if TotalSupplyTable::<T>::get(tapp_id).is_zero() {
			TotalSupplyTable::<T>::remove(tapp_id);
			let item = TAppBondingCurve::<T>::take(tapp_id);
			TAppNames::<T>::remove(item.name);
			TAppTickers::<T>::remove(item.ticker);
		}
	}

	pub(crate) fn distribute_to_investors(tapp_id: TAppId, distributing_amount: BalanceOf<T>) {
		// todo replace total amount with total supply later if calculation result is correct
		let (investors, total_amount) = Self::tapp_investors(tapp_id);
		if !approximately_equals::<T>(
			total_amount,
			TotalSupplyTable::<T>::get(tapp_id),
			CALCULATION_PRECISION.into(),
		) {
			log::error!(
				"distributing calculate total amount error: calculated result is: {:?}, \
				total supply is {:?}",
				total_amount,
				TotalSupplyTable::<T>::get(tapp_id)
			);
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
	pub(crate) fn calculate_given_received_tea_how_much_seller_give_away(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let current_reserve_pool_tea = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply)
			}
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
		};
		ensure!(
			tea_amount <= current_reserve_pool_tea,
			Error::<T>::TAppInsufficientFreeBalance
		);

		let total_supply_after_sell_tapp_token = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance_reverse(
				current_reserve_pool_tea.saturating_sub(tea_amount),
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::pool_balance_reverse(
				current_reserve_pool_tea.saturating_sub(tea_amount),
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance_reverse(
				current_reserve_pool_tea.saturating_sub(tea_amount),
				T::PoolBalanceReversePrecision::get(),
			),
		};
		Ok(total_supply
			.checked_sub(&total_supply_after_sell_tapp_token)
			.ok_or(Error::<T>::SubtractionOverflow)?)
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
		host_performance: Option<Performance>,
		max_allowed_hosts: Option<u32>,
	) -> DispatchResult {
		ensure!(
			(host_performance.is_some() && max_allowed_hosts.is_some())
				|| (host_performance.is_none() && max_allowed_hosts.is_none()),
			Error::<T>::HostPerformanceAndMaxAllowedHostMustBePaired
		);
		if let Some(performance) = host_performance {
			ensure!(
				!performance.is_zero(),
				Error::<T>::PerformanceValueShouldNotBeZero,
			);
		}
		if let Some(max_allowed_hosts) = max_allowed_hosts {
			ensure!(
				!max_allowed_hosts.is_zero(),
				Error::<T>::MaxAllowedHostShouldNotBeZero,
			);
		}

		Ok(())
	}

	pub(crate) fn unhost_tapp(tapp_id: TAppId, cml_id: CmlId) {
		TAppCurrentHosts::<T>::remove(tapp_id, cml_id);
		CmlHostingTApps::<T>::mutate(cml_id, |array| {
			if let Some(index) = array.iter().position(|x| *x == tapp_id) {
				array.remove(index);
			}
		});
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
			total = total.saturating_add(
				TAppBondingCurve::<T>::get(tapp_id)
					.host_performance
					.unwrap_or_default(),
			);
		}
		total
	}

	pub(crate) fn collect_host_cost() {
		TAppBondingCurve::<T>::iter()
			.filter(|(_, tapp)| tapp.host_performance.is_some())
			.for_each(|(id, _)| {
				TAppBondingCurve::<T>::mutate(id, |tapp| {
					tapp.current_cost = tapp.current_cost.saturating_add(
						T::HostCostCoefficient::get()
							.saturating_mul(tapp.host_performance.unwrap().into()),
					);
				})
			});
	}

	pub(crate) fn distribute_to_miners(
		tapp_id: TAppId,
		total_amount: BalanceOf<T>,
	) -> Result<(Vec<T::AccountId>, BalanceOf<T>), DispatchError> {
		let host_count = TAppCurrentHosts::<T>::iter_prefix(tapp_id).count() as u32;
		let each_amount = total_amount / host_count.into();

		let mut miners = Vec::new();
		for (cml_id, _) in TAppCurrentHosts::<T>::iter_prefix(tapp_id) {
			let cml = T::CmlOperation::cml_by_id(&cml_id)?;

			let target = cml.owner().ok_or(Error::<T>::CmlOwnerIsNone)?;
			T::CurrencyOperations::transfer(
				&OperationAccount::<T>::get(),
				target,
				each_amount.clone(),
				ExistenceRequirement::AllowDeath,
			)?;
			miners.push(target.clone());
		}
		Ok((miners, each_amount))
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
	use crate::*;
	use bonding_curve_impl::approximately_equals;

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
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
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
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
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
			let amount =
				BondingCurve::calculate_given_increase_tea_how_much_token_mint(tapp_id, 666666666666);
			assert!(approximately_equals(
				amount.unwrap(),
				1_000_000_000_000,
				1000
			));
			TotalSupplyTable::<Test>::insert(tapp_id, 1_000_000_000_000);
			let amount =
				BondingCurve::calculate_given_increase_tea_how_much_token_mint(tapp_id, 666666666666);
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
