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
			BillingMode::FixedHostingFee(reward_per_1k_performance) => {
				let host_count = TAppCurrentHosts::<T>::iter_prefix(tapp_id).count() as u32;
				let tea_amount = tapp.current_cost.saturating_add(
					reward_per_1k_performance
						.saturating_mul(tapp.host_performance().into())
						.saturating_mul(host_count.into())
						/ 1000u32.into(),
				);

				if let Ok((_, distribute_tea_amount)) =
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

	fn tapp_user_token_asset(who: &Self::AccountId) -> Vec<(u64, Self::Balance)> {
		let mut staking_asset: Vec<(u64, Self::Balance)> =
			AccountTable::<T>::iter_prefix(who).collect();
		TAppReservedBalance::<T>::iter()
			.filter(|(_, account, _)| account.eq(who))
			.for_each(|(tapp_id, _, balances)| {
				let mut total_balance: BalanceOf<T> = Zero::zero();
				for (balance, _) in balances {
					total_balance = total_balance.saturating_add(balance);
				}
				staking_asset.push((tapp_id, total_balance));
			});
		staking_asset
	}

	fn is_cml_hosting(cml_id: u64) -> bool {
		!CmlHostingTApps::<T>::get(cml_id).is_empty()
	}

	fn transfer_reserved_tokens(from: &Self::AccountId, to: &Self::AccountId, cml_id: u64) {
		CmlHostingTApps::<T>::get(cml_id)
			.iter()
			.for_each(|tapp_id| {
				let reserved_balance =
					TAppReservedBalance::<T>::mutate(tapp_id, from, |reserved_balances| {
						if let Some(index) =
							reserved_balances.iter().position(|(_, id)| *id == cml_id)
						{
							return Some(reserved_balances.remove(index));
						}
						None
					});
				if let Some(reserved_balance) = reserved_balance {
					TAppReservedBalance::<T>::mutate(tapp_id, to, |reserved_balances| {
						reserved_balances.push(reserved_balance)
					});
				}
			})
	}

	fn npc_account() -> T::AccountId {
		NPCAccount::<T>::get()
	}

	fn cml_host_tapps(cml_id: CmlId) -> Vec<u64> {
		CmlHostingTApps::<T>::get(cml_id)
	}

	fn try_active_tapp(tapp_id: TAppId) -> bool {
		let host_count = TAppCurrentHosts::<T>::iter_prefix(tapp_id).count();
		if TAppBondingCurve::<T>::get(tapp_id).status == TAppStatus::Pending
			&& host_count >= T::MinTappHostsCount::get() as usize
		{
			let current_block = frame_system::Pallet::<T>::block_number();
			TAppBondingCurve::<T>::mutate(tapp_id, |tapp| {
				tapp.status = TAppStatus::Active(current_block.clone())
			});

			Self::deposit_event(Event::TAppBecomeActived(
				tapp_id,
				current_block,
				host_count as u32,
			));
			return true;
		}
		false
	}

	fn try_deactive_tapp(tapp_id: TAppId) -> bool {
		let host_count = TAppCurrentHosts::<T>::iter_prefix(tapp_id).count();
		match TAppBondingCurve::<T>::get(tapp_id).status {
			TAppStatus::Active(_) => {
				if host_count < T::MinTappHostsCount::get() as usize {
					TAppBondingCurve::<T>::mutate(tapp_id, |tapp| {
						tapp.status = TAppStatus::Pending
					});

					let current_block = frame_system::Pallet::<T>::block_number();
					Self::deposit_event(Event::TAppBecomePending(
						tapp_id,
						current_block,
						host_count as u32,
					));
					return true;
				}
			}
			_ => {}
		}
		false
	}

	fn pay_hosting_penalty(tapp_id: u64, cml_id: u64) {
		match T::CmlOperation::cml_by_id(&cml_id) {
			Ok(cml) => {
				if let Some(owner) = cml.owner() {
					T::CurrencyOperations::slash_reserved(owner, T::HostPledgeAmount::get());
				}
			}
			Err(e) => log::error!("failed to get cml: {:?}", e),
		}

		TAppHostPledge::<T>::mutate(tapp_id, cml_id, |balance| {
			*balance = balance.saturating_sub(T::HostPledgeAmount::get());
		});
	}

	fn can_append_pledge(cml_id: u64) -> bool {
		match T::CmlOperation::cml_by_id(&cml_id) {
			Ok(cml) => {
				if let Some(owner) = cml.owner() {
					return T::CurrencyOperations::free_balance(owner)
						>= T::HostPledgeAmount::get()
							* (CmlHostingTApps::<T>::get(cml_id).len() as u32).into();
				}
				false
			}
			Err(_) => false,
		}
	}

	fn append_pledge(cml_id: u64) -> bool {
		match T::CmlOperation::cml_by_id(&cml_id) {
			Ok(cml) => {
				if let Some(owner) = cml.owner() {
					let mut success = true;
					CmlHostingTApps::<T>::get(cml_id)
						.iter()
						.for_each(|tapp_id| {
							match T::CurrencyOperations::reserve(owner, T::HostPledgeAmount::get())
							{
								Ok(_) => {
									TAppHostPledge::<T>::mutate(tapp_id, cml_id, |amount| {
										*amount = amount.saturating_add(T::HostPledgeAmount::get())
									});
								}
								Err(e) => {
									log::error!("reserve pledge failed: {:?}", e);
									success = false;
								}
							}
						});
					return success;
				}
				false
			}
			Err(_) => false,
		}
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
		let deposit_tea_amount = Self::calculate_buy_amount(Some(tapp_id), tapp_amount, None)?;
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

				if AccountTable::<T>::get(&who, tapp_id).is_zero() {
					AccountTable::<T>::remove(&who, tapp_id);
				}
				deposit_tea_amount
			}
			Err(e) => {
				// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
				log::error!("calculating sell amount failed: {:?}", e);
				return Zero::zero();
			}
		}
	}

	pub fn try_clean_tapp_related(tapp_id: TAppId) -> bool {
		if !TotalSupplyTable::<T>::get(tapp_id).is_zero() {
			return false;
		}

		TotalSupplyTable::<T>::remove(tapp_id);
		let need_remove_keypair: Vec<(T::AccountId, TAppId)> = AccountTable::<T>::iter()
			.filter(|(_, id, _)| *id == tapp_id)
			.map(|(acc, id, _)| (acc, id))
			.collect();
		need_remove_keypair
			.iter()
			.for_each(|(acc, id)| AccountTable::<T>::remove(acc, id));

		let item = TAppBondingCurve::<T>::take(tapp_id);
		TAppNames::<T>::remove(item.name);
		TAppTickers::<T>::remove(item.ticker);
		TAppApprovedLinks::<T>::mutate(item.link, |tapp_info| {
			tapp_info.tapp_id = None;
			tapp_info.creator = None;
		});
		TAppCurrentHosts::<T>::iter_prefix(tapp_id).for_each(|(cml_id, _)| {
			Self::unhost_tapp(tapp_id, cml_id, false);
		});
		TAppReservedBalance::<T>::remove_prefix(tapp_id, None);
		TAppLastActivity::<T>::remove(tapp_id);
		true
	}

	pub(crate) fn user_tapp_total_reserved_balance(
		tapp_id: TAppId,
		who: &T::AccountId,
	) -> BalanceOf<T> {
		let mut account_balance: BalanceOf<T> = Zero::zero();
		TAppReservedBalance::<T>::iter_prefix(tapp_id)
			.filter(|(account, _)| account.eq(who))
			.for_each(|(_, amount_list)| {
				amount_list.iter().for_each(|(balance, _)| {
					account_balance = account_balance.saturating_add(balance.clone());
				});
			});
		account_balance
	}

	pub(crate) fn distribute_to_investors(tapp_id: TAppId, distributing_amount: BalanceOf<T>) {
		let (investors, mut total_amount) = Self::tapp_investors(tapp_id);

		let mut consume_statements: Vec<(T::AccountId, BalanceOf<T>, bool, Option<CmlId>)> =
			Vec::new();

		match TAppBondingCurve::<T>::get(tapp_id).billing_mode {
			BillingMode::FixedHostingToken(_) => {
				TAppReservedBalance::<T>::iter_prefix(tapp_id).for_each(|(_, amount_list)| {
					amount_list
						.iter()
						.filter(|(_, cml_id)| {
							let (is_mining, status) = T::CmlOperation::mining_status(*cml_id);
							is_mining && status.eq(&MinerStatus::Active)
						})
						.for_each(|(balance, _)| {
							total_amount = total_amount.saturating_add(balance.clone());
						});
				});

				TAppReservedBalance::<T>::iter_prefix(tapp_id).for_each(
					|(account, amount_list)| {
						amount_list
							.iter()
							.filter(|(_, cml_id)| {
								let (is_mining, status) = T::CmlOperation::mining_status(*cml_id);
								is_mining && status.eq(&MinerStatus::Active)
							})
							.for_each(|(balance, cml_id)| {
								consume_statements.push((
									account.clone(),
									distributing_amount * (*balance) / total_amount,
									false,
									Some(*cml_id),
								));
							});
					},
				);
			}
			_ => {}
		}

		investors.iter().for_each(|account| {
			AccountTable::<T>::mutate(account, tapp_id, |user_amount| {
				let reward = distributing_amount * (*user_amount) / total_amount;
				*user_amount = user_amount.saturating_add(reward.clone());
				consume_statements.push((account.clone(), reward, true, None));
			});
		});

		consume_statements
			.iter()
			.for_each(|(account, reward, _, cml)| {
				if cml.is_none() {
					return;
				}

				AccountTable::<T>::mutate(account, tapp_id, |user_amount| {
					*user_amount = user_amount.saturating_add(reward.clone());
				});
			});

		Self::deposit_event(Event::TAppConsumeRewardStatements(
			tapp_id,
			consume_statements,
		));

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
		buy_curve_k: Option<u32>,
	) -> Result<BalanceOf<T>, DispatchError> {
		match tapp_id {
			Some(tapp_id) => {
				let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
				let total_supply = TotalSupplyTable::<T>::get(tapp_id);
				Self::calculate_increase_amount_from_raise_curve_total_supply(
					tapp_item.buy_curve_k,
					total_supply,
					tapp_amount,
				)
			}
			None => {
				let curve_k = buy_curve_k.unwrap_or(T::DefaultBuyCurveTheta::get());
				Self::calculate_increase_amount_from_raise_curve_total_supply(
					curve_k,
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
			tapp_item.sell_curve_k,
			total_supply,
			tapp_amount,
		)
	}

	pub(crate) fn calculate_increase_amount_from_raise_curve_total_supply(
		curve_k: u32,
		total_supply: BalanceOf<T>,
		tapp_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let current_curve = UnsignedSquareRoot::new(curve_k);
		let current_pool_balance = current_curve.pool_balance(total_supply);

		let after_buy_pool_balance = current_curve.pool_balance(
			total_supply
				.checked_add(&tapp_amount)
				.ok_or(Error::<T>::AddOverflow)?,
		);
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
		let buy_curve = UnsignedSquareRoot::new(tapp_item.buy_curve_k);
		let current_buy_area_tea_amount = buy_curve.pool_balance(total_supply);
		let after_increase_tea_amount = current_buy_area_tea_amount
			.checked_add(&tea_amount)
			.ok_or(Error::<T>::AddOverflow)?;
		let total_supply_after_increase = buy_curve.pool_balance_reverse(
			after_increase_tea_amount,
			T::PoolBalanceReversePrecision::get(),
		);
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

		let sell_curve = UnsignedSquareRoot::new(tapp_item.sell_curve_k);
		let current_pool_balance = sell_curve.pool_balance(total_supply);
		let after_sell_pool_balance =
			sell_curve.pool_balance(total_supply.saturating_sub(tapp_amount));
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
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
		let sell_curve = UnsignedSquareRoot::new(tapp_item.sell_curve_k);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let current_reserve_pool_tea = sell_curve.pool_balance(total_supply);

		let pay_amount = if current_reserve_pool_tea < tea_amount {
			current_reserve_pool_tea
		} else {
			tea_amount
		};

		let total_supply_after_sell_tapp_token = sell_curve.pool_balance_reverse(
			current_reserve_pool_tea.saturating_sub(pay_amount),
			T::PoolBalanceReversePrecision::get(),
		);
		Ok((
			total_supply
				.checked_sub(&total_supply_after_sell_tapp_token)
				.ok_or(Error::<T>::SubtractionOverflow)?,
			pay_amount,
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
		reward_per_1k_performance: &Option<BalanceOf<T>>,
		stake_token_amount: &Option<BalanceOf<T>>,
	) -> DispatchResult {
		ensure!(
			max_allowed_hosts >= T::MinTappHostsCount::get(),
			Error::<T>::MaxAllowedHostShouldLargerEqualThanMinAllowedHosts,
		);
		ensure!(
			!(stake_token_amount.is_some() && reward_per_1k_performance.is_some()),
			Error::<T>::StakeTokenAmountAndRewardPerPerformanceCannotBothExist
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
				reward_per_1k_performance.is_some(),
				Error::<T>::RewardPerPerformanceIsNoneInFixedFeeMode
			);
			ensure!(
				!reward_per_1k_performance.unwrap().is_zero(),
				Error::<T>::RewardPerPerformanceShouldNotBeZero
			);
		}

		Ok(())
	}

	pub(crate) fn unhost_tapp(tapp_id: TAppId, cml_id: CmlId, cml_dead: bool) -> bool {
		TAppCurrentHosts::<T>::remove(tapp_id, cml_id);

		match TAppBondingCurve::<T>::get(tapp_id).billing_mode {
			BillingMode::FixedHostingToken(_) => {
				// todo improve me
				TAppReservedBalance::<T>::iter_prefix(tapp_id).for_each(
					|(account, balance_list)| {
						let updated_list: Vec<(BalanceOf<T>, CmlId)> = balance_list
							.iter()
							.filter(|(_, cml)| *cml != cml_id)
							.map(|(balance, cml)| (balance.clone(), *cml))
							.collect();
						TAppReservedBalance::<T>::insert(tapp_id, account, updated_list);
					},
				);
			}
			_ => {}
		}

		if cml_dead {
			CmlHostingTApps::<T>::remove(cml_id);
		} else {
			CmlHostingTApps::<T>::mutate(cml_id, |array| {
				if let Some(index) = array.iter().position(|x| *x == tapp_id) {
					array.remove(index);
				}
			});
		}

		Self::try_deactive_tapp(tapp_id)
	}

	pub(crate) fn unhost_last_tapp(cml_id: CmlId) -> Option<TAppId> {
		if let Some(last_tapp) = CmlHostingTApps::<T>::get(cml_id).last() {
			Self::unhost_tapp(*last_tapp, cml_id, false);
			return Some(*last_tapp);
		}
		None
	}

	pub(crate) fn arrange_host() {
		let current_block = frame_system::Pallet::<T>::block_number();
		Self::try_clean_died_host_machines(&current_block);

		let mining_cmls = T::CmlOperation::current_mining_cmls();

		let mut unhosted_list = Vec::new();
		mining_cmls.iter().for_each(|cml_id| {
			if T::CmlOperation::is_cml_over_max_suspend_height(*cml_id, &current_block) {
				CmlHostingTApps::<T>::get(cml_id)
					.iter()
					.for_each(|tapp_id| {
						Self::unhost_tapp(*tapp_id, *cml_id, false);
					});
			}

			let (current_performance, _) =
				T::CmlOperation::miner_performance(*cml_id, &current_block);
			while Self::cml_total_used_performance(*cml_id) > current_performance.unwrap_or(0) {
				if let Some(tapp_id) = Self::unhost_last_tapp(*cml_id) {
					unhosted_list.push((tapp_id, *cml_id));
				}
			}
		});

		Self::deposit_event(Event::TAppsAutoUnhosted(unhosted_list));
	}

	pub(crate) fn try_clean_died_host_machines(current_block: &T::BlockNumber) {
		TAppBondingCurve::<T>::iter().for_each(|(tapp_id, _)| {
			TAppCurrentHosts::<T>::iter_prefix(tapp_id).for_each(|(cml_id, _)| {
				match T::CmlOperation::cml_by_id(&cml_id) {
					Ok(cml) => {
						if cml.should_dead(current_block) || !cml.is_mining() {
							Self::unhost_tapp(tapp_id, cml_id, true);
						}
					}
					Err(_) => {
						// error means cml is already dead and removed from cml store
						Self::unhost_tapp(tapp_id, cml_id, true);
					}
				}
			})
		});
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
			.filter(|(_, tapp)| {
				let fix_fee_mode = match tapp.billing_mode {
					BillingMode::FixedHostingFee(_) => true,
					_ => false,
				};
				fix_fee_mode && tapp.status != TAppStatus::Pending
			})
			.for_each(|(id, tapp)| match tapp.billing_mode {
				BillingMode::FixedHostingFee(reward_per_1k_performance) => {
					Self::accumulate_tapp_cost(id, reward_per_1k_performance);
					Self::expense_inner(id)
				}
				_ => {}
			});
	}

	pub(crate) fn accumulate_tapp_cost(tapp_id: TAppId, reward_per_1k_performance: BalanceOf<T>) {
		TAppBondingCurve::<T>::mutate(tapp_id, |tapp| {
			let host_count = TAppCurrentHosts::<T>::iter_prefix(tapp_id).count() as u32;
			tapp.current_cost = tapp.current_cost.saturating_add(
				reward_per_1k_performance
					.saturating_mul(tapp.host_performance().into())
					.saturating_mul(host_count.into())
					/ 1000u32.into(),
			);
		});
	}

	pub(crate) fn distribute_to_miners(
		tapp_id: TAppId,
		total_amount: BalanceOf<T>,
		do_transfer: bool,
	) -> Result<Vec<(T::AccountId, CmlId, BalanceOf<T>)>, DispatchError> {
		let host_count = TAppCurrentHosts::<T>::iter_prefix(tapp_id)
			.filter(|(cml_id, _)| {
				let (is_mining, status) = T::CmlOperation::mining_status(*cml_id);
				is_mining && status.eq(&MinerStatus::Active)
			})
			.count() as u32;
		ensure!(
			!host_count.is_zero(),
			Error::<T>::NoHostingToDistributeMiner
		);

		let each_amount = total_amount / host_count.into();

		let mut tapp_statements = Vec::new();
		for (cml_id, _) in TAppCurrentHosts::<T>::iter_prefix(tapp_id) {
			let (is_mining, status) = T::CmlOperation::mining_status(cml_id);
			if !is_mining || !status.eq(&MinerStatus::Active) {
				continue;
			}

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
			Ok((withdraw_tapp_amount, distribute_tea_amount)) => {
				match Self::distribute_to_miners(tapp_id, distribute_tea_amount, true) {
					Ok(tapp_statements) => {
						Self::collect_with_investors(tapp_id, withdraw_tapp_amount);
						TAppBondingCurve::<T>::mutate(tapp_id, |tapp_item| {
							tapp_item.current_cost = Zero::zero();
						});

						let (buy_price, sell_price) = Self::query_price(tapp_id);
						let is_fix_token_mode =
							match TAppBondingCurve::<T>::get(tapp_id).billing_mode {
								BillingMode::FixedHostingToken(_) => true,
								_ => false,
							};
						Self::deposit_event(Event::TAppExpense(
							tapp_id,
							tapp_statements,
							buy_price,
							sell_price,
							TotalSupplyTable::<T>::get(tapp_id),
							is_fix_token_mode,
						));
					}
					Err(e) => {
						// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
						log::error!("transfer free balance failed: {:?}", e);
					}
				}

				Self::try_bankrupt_tapp(tapp_id);
			}
			Err(e) => {
				// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
				log::error!("calculation failed: {:?}", e);
			}
		}
	}

	pub(crate) fn try_bankrupt_tapp(tapp_id: TAppId) {
		if Self::try_clean_tapp_related(tapp_id) {
			Self::deposit_event(Event::TAppBankrupted(tapp_id));
		}
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
	use frame_support::{assert_noop, assert_ok};
	use pallet_cml::{CmlOperation, CmlStore, UserCmlStore, CML};

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
				b"miner_ip".to_vec(),
				b"orbitdb id".to_vec(),
			));
			assert_ok!(Cml::start_mining(
				Origin::signed(miner),
				cml_id2,
				[2u8; 32],
				b"miner_ip2".to_vec(),
				b"orbitdb id".to_vec(),
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
				HOST_COST_COEFFICIENT.saturating_mul(performance.into()) / 1000
			);

			// Add second host, the cost should be 1000*HostCostCoefficient*2
			TAppBondingCurve::<Test>::mutate(tapp_id, |tapp_item| tapp_item.current_cost = 0);
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id2, tapp_id));
			BondingCurve::accumulate_tapp_cost(tapp_id, HOST_COST_COEFFICIENT);
			assert_eq!(
				TAppBondingCurve::<Test>::get(tapp_id).current_cost,
				HOST_COST_COEFFICIENT.saturating_mul((performance * 2).into()) / 1000
			);

			frame_system::Pallet::<Test>::set_block_number(1001);
			// remove the first host, the cost should be 1000*HostCostCoefficient
			TAppBondingCurve::<Test>::mutate(tapp_id, |tapp_item| tapp_item.current_cost = 0);
			assert_ok!(BondingCurve::unhost(Origin::signed(miner), cml_id, tapp_id));
			BondingCurve::accumulate_tapp_cost(tapp_id, HOST_COST_COEFFICIENT);
			assert_eq!(
				TAppBondingCurve::<Test>::get(tapp_id).current_cost,
				HOST_COST_COEFFICIENT.saturating_mul(performance.into()) / 1000
			);
		})
	}

	#[test]
	fn try_clean_tapp_related_works() {
		new_test_ext().execute_with(|| {
			EnableUserCreateTApp::<Test>::set(true);

			let owner1 = 11;
			let owner2 = 12;
			let owner3 = 13;
			let user1 = 21;
			let user2 = 22;
			let user3 = 23;
			let miner1 = 31;
			let miner2 = 32;
			<Test as Config>::Currency::make_free_balance_be(&owner1, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&owner2, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&owner3, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&user1, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&user2, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&user3, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&miner1, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&miner2, 100000000);

			let npc = NPCAccount::<Test>::get();
			let link1 = b"https://teaproject.org".to_vec();
			let link2 = b"https://teaproject2.org".to_vec();
			let link3 = b"https://teaproject3.org".to_vec();
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				link1.clone(),
				"test description".into(),
				Some(owner1),
			));
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				link2.clone(),
				"test description".into(),
				None,
			));
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				link3.clone(),
				"test description".into(),
				None,
			));

			let name1 = b"test name1".to_vec();
			let ticker1 = b"tea1".to_vec();
			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(owner1),
				name1.clone(),
				ticker1.clone(),
				1_000_000,
				b"test detail".to_vec(),
				link1.clone(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			));
			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(owner1),
				b"test name2".to_vec(),
				b"tea2".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				link2,
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			));
			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(owner1),
				b"test name3".to_vec(),
				b"tea3".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				link3,
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			));

			let tapp_id = 1;
			assert_ok!(BondingCurve::buy_token(
				Origin::signed(user1),
				tapp_id,
				1000
			));
			assert_ok!(BondingCurve::buy_token(
				Origin::signed(user2),
				tapp_id,
				1000
			));
			assert_ok!(BondingCurve::buy_token(
				Origin::signed(user3),
				tapp_id,
				1000
			));

			let cml_id1 = 11;
			let cml_id2 = 22;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id1, 100, 10000));
			let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 100, 10000));
			UserCmlStore::<Test>::insert(miner1, cml_id1, ());
			UserCmlStore::<Test>::insert(miner2, cml_id2, ());
			CmlStore::<Test>::insert(cml_id1, cml);
			CmlStore::<Test>::insert(cml_id2, cml2);
			assert_ok!(Cml::start_mining(
				Origin::signed(miner1),
				cml_id1,
				[1u8; 32],
				b"miner_ip".to_vec(),
				b"orbitdb id".to_vec(),
			));
			assert_ok!(Cml::start_mining(
				Origin::signed(miner2),
				cml_id2,
				[2u8; 32],
				b"miner_ip2".to_vec(),
				b"orbitdb id".to_vec(),
			));
			assert_ok!(BondingCurve::host(Origin::signed(miner1), cml_id1, tapp_id));
			assert_ok!(BondingCurve::host(Origin::signed(miner2), cml_id2, tapp_id));

			TAppLastActivity::<Test>::insert(tapp_id, (3, 1));

			assert!(!TotalSupplyTable::<Test>::get(tapp_id).is_zero());
			assert!(AccountTable::<Test>::contains_key(owner1, tapp_id));
			assert!(AccountTable::<Test>::contains_key(user1, tapp_id));
			assert!(AccountTable::<Test>::contains_key(user2, tapp_id));
			assert!(AccountTable::<Test>::contains_key(user3, tapp_id));
			assert!(TAppBondingCurve::<Test>::contains_key(tapp_id));
			assert!(TAppNames::<Test>::contains_key(&name1));
			assert!(TAppTickers::<Test>::contains_key(&ticker1));
			assert_eq!(TAppCurrentHosts::<Test>::iter_prefix(tapp_id).count(), 2);
			assert_eq!(CmlHostingTApps::<Test>::get(cml_id1).len(), 1);
			assert_eq!(CmlHostingTApps::<Test>::get(cml_id2).len(), 1);
			assert!(TAppApprovedLinks::<Test>::get(&link1).tapp_id.is_some());
			assert!(TAppApprovedLinks::<Test>::get(&link1).creator.is_some());
			assert!(TAppLastActivity::<Test>::contains_key(tapp_id));
			assert_eq!(TAppReservedBalance::<Test>::iter_prefix(tapp_id).count(), 2);

			TotalSupplyTable::<Test>::insert(tapp_id, 0);
			BondingCurve::try_clean_tapp_related(tapp_id);

			assert!(!AccountTable::<Test>::contains_key(owner1, tapp_id));
			assert!(!AccountTable::<Test>::contains_key(user1, tapp_id));
			assert!(!AccountTable::<Test>::contains_key(user2, tapp_id));
			assert!(!AccountTable::<Test>::contains_key(user3, tapp_id));
			assert!(!TAppBondingCurve::<Test>::contains_key(tapp_id));
			assert!(!TAppNames::<Test>::contains_key(&name1));
			assert!(!TAppTickers::<Test>::contains_key(&ticker1));
			assert_eq!(TAppCurrentHosts::<Test>::iter_prefix(tapp_id).count(), 0);
			assert_eq!(CmlHostingTApps::<Test>::get(cml_id1).len(), 0);
			assert_eq!(CmlHostingTApps::<Test>::get(cml_id2).len(), 0);
			assert!(!TAppApprovedLinks::<Test>::get(&link1).tapp_id.is_some());
			assert!(!TAppApprovedLinks::<Test>::get(&link1).creator.is_some());
			assert!(!TAppLastActivity::<Test>::contains_key(tapp_id));
			assert_eq!(TAppReservedBalance::<Test>::iter_prefix(tapp_id).count(), 0);
		})
	}

	#[test]
	fn expense_to_bankrust_works() {
		new_test_ext().execute_with(|| {
			EnableUserCreateTApp::<Test>::set(true);
			let user1 = 1;
			let user2 = 2;
			let user3 = 3;
			let user4 = 4;
			let miner1 = 5;
			let miner2 = 6;
			let tapp_amount2 = 2_000_000;
			let tapp_amount3 = 4_000_000;
			<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
			<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);
			<Test as Config>::Currency::make_free_balance_be(&user3, DOLLARS);
			<Test as Config>::Currency::make_free_balance_be(&user4, DOLLARS);
			<Test as Config>::Currency::make_free_balance_be(&miner1, DOLLARS);
			<Test as Config>::Currency::make_free_balance_be(&miner2, DOLLARS);

			let npc = NPCAccount::<Test>::get();
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				"https://teaproject.org".into(),
				"test description".into(),
				None,
			));
			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(user1),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1_000_000),
				None,
				None,
			));

			let cml_id1 = 11;
			let cml_id2 = 22;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id1, 100, 10000));
			let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 100, 10000));
			UserCmlStore::<Test>::insert(miner1, cml_id1, ());
			UserCmlStore::<Test>::insert(miner2, cml_id2, ());
			CmlStore::<Test>::insert(cml_id1, cml);
			CmlStore::<Test>::insert(cml_id2, cml2);
			assert_ok!(Cml::start_mining(
				Origin::signed(miner1),
				cml_id1,
				[1u8; 32],
				b"miner_ip".to_vec(),
				b"orbitdb id".to_vec(),
			));
			assert_ok!(Cml::start_mining(
				Origin::signed(miner2),
				cml_id2,
				[2u8; 32],
				b"miner_ip2".to_vec(),
				b"orbitdb id".to_vec(),
			));

			let tapp_id = 1;
			assert_ok!(BondingCurve::buy_token(
				Origin::signed(user2),
				tapp_id,
				tapp_amount2
			));
			assert_ok!(BondingCurve::buy_token(
				Origin::signed(user3),
				tapp_id,
				tapp_amount3
			));
			assert_ok!(BondingCurve::host(Origin::signed(miner1), cml_id1, tapp_id));
			assert_ok!(BondingCurve::host(Origin::signed(miner2), cml_id2, tapp_id));

			TAppBondingCurve::<Test>::mutate(tapp_id, |tapp_item| {
				tapp_item.current_cost = 1000000 * DOLLARS
			});
			BondingCurve::expense_inner(tapp_id);

			assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), 0);

			assert!(!AccountTable::<Test>::contains_key(user1, tapp_id));
			assert!(!AccountTable::<Test>::contains_key(user2, tapp_id));
			assert!(!AccountTable::<Test>::contains_key(user3, tapp_id));
			assert!(!AccountTable::<Test>::contains_key(miner1, tapp_id));
			assert!(!AccountTable::<Test>::contains_key(miner2, tapp_id));
			assert!(!TAppBondingCurve::<Test>::contains_key(tapp_id));
			assert_eq!(TAppCurrentHosts::<Test>::iter_prefix(tapp_id).count(), 0);
			assert_eq!(CmlHostingTApps::<Test>::get(cml_id1).len(), 0);
			assert_eq!(CmlHostingTApps::<Test>::get(cml_id2).len(), 0);
			assert!(!TAppLastActivity::<Test>::contains_key(tapp_id));
			assert_eq!(TAppReservedBalance::<Test>::iter_prefix(tapp_id).count(), 0);
		})
	}

	#[test]
	fn clean_died_host_machines_works() {
		new_test_ext().execute_with(|| {
			EnableUserCreateTApp::<Test>::set(true);
			let miner = 2;
			let tapp_owner = 1;
			<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

			let cml_id = 11;
			let cml_id2 = 22;
			let cml_id4 = 44;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
			let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 1000, 10000));
			let cml4 = CML::from_genesis_seed(seed_from_lifespan(cml_id4, 1000, 10000));
			UserCmlStore::<Test>::insert(miner, cml_id, ());
			UserCmlStore::<Test>::insert(miner, cml_id2, ());
			UserCmlStore::<Test>::insert(miner, cml_id4, ());
			CmlStore::<Test>::insert(cml_id, cml);
			CmlStore::<Test>::insert(cml_id2, cml2);
			CmlStore::<Test>::insert(cml_id4, cml4);

			assert_ok!(Cml::start_mining(
				Origin::signed(miner),
				cml_id,
				[1u8; 32],
				b"miner_ip".to_vec(),
				b"orbitdb id".to_vec(),
			));
			assert_ok!(Cml::start_mining(
				Origin::signed(miner),
				cml_id2,
				[2u8; 32],
				b"miner_ip2".to_vec(),
				b"orbitdb id".to_vec(),
			));
			assert_ok!(Cml::start_mining(
				Origin::signed(miner),
				cml_id4,
				[4u8; 32],
				b"miner_ip4".to_vec(),
				b"orbitdb id".to_vec(),
			));

			assert_ok!(create_default_tapp(tapp_owner));

			let tapp_id = 1;
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id2, tapp_id));
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id4, tapp_id));
			assert_eq!(TAppReservedBalance::<Test>::get(tapp_id, miner).len(), 3);

			let cml_id3 = 33;
			TAppCurrentHosts::<Test>::insert(tapp_id, cml_id3, 10);

			assert_noop!(
				Cml::stop_mining(Origin::signed(miner), cml_id4, [4u8; 32]),
				pallet_cml::Error::<Test>::CannotStopMiningWhenHostingTApp
			);
			BondingCurve::try_clean_died_host_machines(&200);

			assert_eq!(TAppCurrentHosts::<Test>::iter_prefix(tapp_id).count(), 2);
			assert!(!TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id));
			assert!(TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id2));
			assert!(!TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id3));
			assert!(TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id4));
			assert!(!CmlHostingTApps::<Test>::contains_key(cml_id));
			assert!(CmlHostingTApps::<Test>::contains_key(cml_id2));
			assert!(!CmlHostingTApps::<Test>::contains_key(cml_id3));
			assert!(CmlHostingTApps::<Test>::contains_key(cml_id4));
			assert_eq!(TAppReservedBalance::<Test>::get(tapp_id, miner).len(), 2);
			assert_eq!(
				TAppReservedBalance::<Test>::get(tapp_id, miner)[0],
				(1000, cml_id2)
			);
			assert_eq!(
				TAppReservedBalance::<Test>::get(tapp_id, miner)[1],
				(1000, cml_id4)
			);
		})
	}

	#[test]
	fn arrange_host_works() {
		new_test_ext().execute_with(|| {
			EnableUserCreateTApp::<Test>::set(true);
			let miner = 2;
			let tapp_owner = 1;
			<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&miner, 1000000);

			let cml_id = 11;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 2000));
			UserCmlStore::<Test>::insert(miner, cml_id, ());
			CmlStore::<Test>::insert(cml_id, cml);

			assert_ok!(Cml::start_mining(
				Origin::signed(miner),
				cml_id,
				[1u8; 32],
				b"miner_ip".to_vec(),
				b"orbitdb id".to_vec(),
			));

			frame_system::Pallet::<Test>::set_block_number(40);
			let (performance, _) = Cml::miner_performance(cml_id, &40);
			assert_eq!(performance, Some(2000));

			assert_ok!(create_default_tapp(tapp_owner));
			let npc = NPCAccount::<Test>::get();
			let link = b"https://teaproject2.org".to_vec();
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				link.clone(),
				"test description".into(),
				None,
			));
			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(tapp_owner),
				b"test name2".to_vec(),
				b"tea2".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				link,
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			));

			let tapp_id = 1;
			let tapp_id2 = 2;
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id2));

			assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 2);

			frame_system::Pallet::<Test>::set_block_number(60);
			let (performance, _) = Cml::miner_performance(cml_id, &60);
			assert_eq!(performance, Some(1400));

			BondingCurve::arrange_host();

			assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 1);
			assert_eq!(CmlHostingTApps::<Test>::get(cml_id)[0], tapp_id);
		})
	}

	#[test]
	fn arrange_host_works_when_cml_suspend_for_a_long_time() {
		new_test_ext().execute_with(|| {
			EnableUserCreateTApp::<Test>::set(true);
			let miner = 2;
			let tapp_owner = 1;
			<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&miner, 1000000);

			let cml_id = 11;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 10000, 2000));
			UserCmlStore::<Test>::insert(miner, cml_id, ());
			CmlStore::<Test>::insert(cml_id, cml);

			assert_ok!(Cml::start_mining(
				Origin::signed(miner),
				cml_id,
				[1u8; 32],
				b"miner_ip".to_vec(),
				b"orbitdb id".to_vec(),
			));

			frame_system::Pallet::<Test>::set_block_number(4000);
			let (performance, _) = Cml::miner_performance(cml_id, &4000);
			assert_eq!(performance, Some(2000));

			assert_ok!(create_default_tapp(tapp_owner));
			let npc = NPCAccount::<Test>::get();
			let link = b"https://teaproject2.org".to_vec();
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				link.clone(),
				"test description".into(),
				None,
			));
			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(tapp_owner),
				b"test name2".to_vec(),
				b"tea2".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				link,
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			));

			let tapp_id = 1;
			let tapp_id2 = 2;
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id2));

			assert_ok!(Cml::suspend_mining(Origin::signed(npc), cml_id));

			frame_system::Pallet::<Test>::set_block_number(4000 + HOST_ARRANGE_DURATION + 1);
			let (performance, _) =
				Cml::miner_performance(cml_id, &(4000 + HOST_ARRANGE_DURATION + 1));
			assert_eq!(performance, Some(1800));

			assert!(Cml::is_cml_over_max_suspend_height(
				cml_id,
				&(4000 + HOST_ARRANGE_DURATION + 1)
			));
			BondingCurve::arrange_host();

			assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 0);
		})
	}

	#[test]
	fn transfer_reserved_tokens_works() {
		new_test_ext().execute_with(|| {
			EnableUserCreateTApp::<Test>::set(true);
			let miner = 2;
			let bid_winner = 3;
			let tapp_owner = 1;
			<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
			<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

			let cml_id = 11;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 100000));
			UserCmlStore::<Test>::insert(miner, cml_id, ());
			CmlStore::<Test>::insert(cml_id, cml);

			assert_ok!(Cml::start_mining(
				Origin::signed(miner),
				cml_id,
				[1u8; 32],
				b"miner_ip".to_vec(),
				b"orbitdb id".to_vec(),
			));

			let npc = NPCAccount::<Test>::get();
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				"https://teaproject.org".into(),
				"test description".into(),
				None,
			));
			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(tapp_owner),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				1,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			));

			let npc = NPCAccount::<Test>::get();
			let link2 = b"https://tearust.org".to_vec();
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				link2.clone(),
				"test description".into(),
				None,
			));
			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(tapp_owner),
				b"test name2".to_vec(),
				b"tea2".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				link2,
				1,
				TAppType::Twitter,
				true,
				None,
				Some(2000),
				None,
				None,
			));

			let tapp_id = 1;
			let tapp_id2 = 2;
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));
			assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id2));

			assert_eq!(
				TAppReservedBalance::<Test>::get(tapp_id, miner)[0],
				(1000, cml_id)
			);
			assert_eq!(
				TAppReservedBalance::<Test>::get(tapp_id2, miner)[0],
				(2000, cml_id)
			);

			BondingCurve::transfer_reserved_tokens(&miner, &bid_winner, cml_id);
			assert_eq!(TAppReservedBalance::<Test>::get(tapp_id, miner).len(), 0);
			assert_eq!(TAppReservedBalance::<Test>::get(tapp_id2, miner).len(), 0);
			assert_eq!(
				TAppReservedBalance::<Test>::get(tapp_id, bid_winner)[0],
				(1000, cml_id)
			);
			assert_eq!(
				TAppReservedBalance::<Test>::get(tapp_id2, bid_winner)[0],
				(2000, cml_id)
			);
		})
	}

	#[test]
	fn pay_hosting_penalty_works() {
		new_test_ext().execute_with(|| {
			let miner = 2;
			let miner_initial_amount = 10000;
			<Test as Config>::Currency::make_free_balance_be(&miner, miner_initial_amount);

			let cml_id = 11;
			let tapp_id = 22;
			let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 100000));
			cml.set_owner(&miner);
			UserCmlStore::<Test>::insert(miner, cml_id, ());
			CmlStore::<Test>::insert(cml_id, cml);

			Utils::reserve(&miner, HOST_PLEDGE_AMOUNT).unwrap();
			TAppHostPledge::<Test>::insert(tapp_id, cml_id, HOST_PLEDGE_AMOUNT);

			assert_eq!(Utils::reserved_balance(&miner), HOST_PLEDGE_AMOUNT);
			BondingCurve::pay_hosting_penalty(tapp_id, cml_id);

			assert_eq!(Utils::reserved_balance(&miner), 0);
			assert_eq!(TAppHostPledge::<Test>::get(tapp_id, cml_id), 0);
		})
	}

	#[test]
	fn append_pledge_works() {
		new_test_ext().execute_with(|| {
			let miner = 2;
			let miner_initial_amount = 500;
			<Test as Config>::Currency::make_free_balance_be(&miner, miner_initial_amount);

			let cml_id = 11;
			let tapp_id = 22;
			let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 100000));
			cml.set_owner(&miner);
			UserCmlStore::<Test>::insert(miner, cml_id, ());
			CmlStore::<Test>::insert(cml_id, cml);

			CmlHostingTApps::<Test>::insert(cml_id, vec![1, 2, 3]);
			assert_eq!(Utils::reserved_balance(&miner), 0);
			assert_eq!(TAppHostPledge::<Test>::get(tapp_id, cml_id), 0);

			assert!(BondingCurve::can_append_pledge(cml_id));
			BondingCurve::append_pledge(cml_id);
			assert_eq!(Utils::reserved_balance(&miner), HOST_PLEDGE_AMOUNT * 3);
			assert_eq!(
				Utils::free_balance(&miner),
				miner_initial_amount - HOST_PLEDGE_AMOUNT * 3
			);
			assert_eq!(TAppHostPledge::<Test>::get(1, cml_id), HOST_PLEDGE_AMOUNT);
			assert_eq!(TAppHostPledge::<Test>::get(2, cml_id), HOST_PLEDGE_AMOUNT);
			assert_eq!(TAppHostPledge::<Test>::get(3, cml_id), HOST_PLEDGE_AMOUNT);

			assert!(!BondingCurve::can_append_pledge(cml_id));
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
