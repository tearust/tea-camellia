use super::*;
use pallet_cml::{SeedProperties, TreeProperties};

impl<T: bonding_curve::Config> bonding_curve::Pallet<T> {
	pub fn query_price(tapp_id: TAppId) -> (BalanceOf<T>, BalanceOf<T>) {
		let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let buy_curve = UnsignedSquareRoot::new(tapp_item.buy_curve_k);
		let sell_curve = UnsignedSquareRoot::new(tapp_item.sell_curve_k);
		let buy_price = buy_curve.buy_price(total_supply);
		let sell_price = sell_curve.buy_price(total_supply);
		(buy_price, sell_price)
	}

	pub fn estimate_required_tea_when_buy(
		tapp_id: Option<TAppId>,
		tapp_amount: BalanceOf<T>,
		buy_curve_k: Option<u32>,
	) -> BalanceOf<T> {
		match Self::calculate_buy_amount(tapp_id, tapp_amount, buy_curve_k) {
			Ok(result) => result,
			Err(e) => {
				log::error!("calculation failed: {:?}", e);
				Zero::zero()
			}
		}
	}

	pub fn estimate_receive_tea_when_sell(
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		match Self::calculate_sell_amount(tapp_id, tapp_amount) {
			Ok(balance) => balance,
			Err(e) => {
				log::error!("calculation failed: {:?}", e);
				Zero::zero()
			}
		}
	}

	pub fn estimate_receive_token_when_buy(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		match Self::calculate_given_increase_tea_how_much_token_mint(tapp_id, tea_amount) {
			Ok(result) => result,
			Err(e) => {
				log::error!("calculation failed: {:?}", e);
				Zero::zero()
			}
		}
	}

	pub fn estimate_required_token_when_sell(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		match Self::calculate_given_received_tea_how_much_seller_give_away(tapp_id, tea_amount) {
			Ok((balance, _)) => balance,
			Err(e) => {
				log::error!("calculate failed: {:?}", e);
				Zero::zero()
			}
		}
	}

	/// Returned item fields:
	/// - TApp Name
	/// - TApp Id
	/// - TApp Ticker
	/// - Total supply
	/// - Token buy price
	/// - Token sell price
	/// - Owner
	/// - Detail
	/// - Link
	/// - Host performance requirement (return zero if is none)
	/// - (current hosts (return zero if is none), max hosts (return zero if is none))
	/// - active block number (return none if not active)
	pub fn list_tapps(
		active_only: bool,
	) -> Vec<(
		Vec<u8>,
		TAppId,
		Vec<u8>,
		BalanceOf<T>,
		BalanceOf<T>,
		BalanceOf<T>,
		T::AccountId,
		Vec<u8>,
		Vec<u8>,
		Performance,
		(u32, u32),
		Option<T::BlockNumber>,
	)> {
		TAppBondingCurve::<T>::iter()
			.filter(|(_, item)| match active_only {
				true => item.status != TAppStatus::Pending,
				false => true,
			})
			.map(|(id, item)| {
				let (buy_price, sell_price) = Self::query_price(id);
				let total_supply = TotalSupplyTable::<T>::get(id);

				let active_height: Option<T::BlockNumber> = match item.status {
					TAppStatus::Active(height) => Some(height),
					_ => None,
				};
				let host_performance = item.host_performance();
				(
					item.name,
					id,
					item.ticker,
					total_supply,
					buy_price,
					sell_price,
					item.owner,
					item.detail,
					item.link,
					host_performance,
					(
						TAppCurrentHosts::<T>::iter_prefix(item.id).count() as u32,
						item.max_allowed_hosts,
					),
					active_height,
				)
			})
			.collect()
	}

	/// Returned item fields:
	/// - TApp Name
	/// - TApp Id
	/// - TApp Ticker
	/// - 1. User holding tokens (inverstor side only, not including mining reserved balance)
	///   2. User reserved tokens (mining reserved balance only)
	/// - Token sell price
	/// - Owner
	/// - Detail
	/// - Link
	/// - Host performance requirement (return zero if is none)
	/// - current hosts (return zero if is none)
	/// - max hosts (return zero if is none)
	/// - Total supply
	pub fn list_user_assets(
		who: &T::AccountId,
	) -> Vec<(
		Vec<u8>,
		TAppId,
		Vec<u8>,
		(BalanceOf<T>, BalanceOf<T>),
		BalanceOf<T>,
		T::AccountId,
		Vec<u8>,
		Vec<u8>,
		Performance,
		u32,
		u32,
		BalanceOf<T>,
	)> {
		AccountTable::<T>::iter_prefix(who)
			.filter(|(id, amount)| {
				!amount.is_zero() || !Self::user_tapp_total_reserved_balance(*id, who).is_zero()
			})
			.map(|(id, amount)| {
				let (_, sell_price) = Self::query_price(id);
				let item = TAppBondingCurve::<T>::get(id);
				let total_supply = TotalSupplyTable::<T>::get(id);

				let host_performance = item.host_performance();
				(
					item.name,
					id,
					item.ticker,
					(amount, Self::user_tapp_total_reserved_balance(id, who)),
					sell_price,
					item.owner,
					item.detail,
					item.link,
					host_performance,
					TAppCurrentHosts::<T>::iter_prefix(item.id).count() as u32,
					item.max_allowed_hosts,
					total_supply,
				)
			})
			.collect()
	}

	/// Returned item fields:
	/// - TApp Name
	/// - TApp Id
	/// - TApp Ticker
	/// - Owner
	/// - Detail
	/// - Link
	/// - Host performance requirement
	/// - current hosts
	/// - max hosts
	/// - Total supply
	/// - Token buy price
	/// - Token sell price
	pub fn tapp_details(
		tapp_id: TAppId,
	) -> (
		Vec<u8>,
		TAppId,
		Vec<u8>,
		T::AccountId,
		Vec<u8>,
		Vec<u8>,
		Performance,
		u32,
		u32,
		BalanceOf<T>,
		BalanceOf<T>,
		BalanceOf<T>,
	) {
		let item = TAppBondingCurve::<T>::get(tapp_id);
		let (buy_price, sell_price) = Self::query_price(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);

		let host_performance = item.host_performance();
		(
			item.name,
			tapp_id,
			item.ticker,
			item.owner,
			item.detail,
			item.link,
			host_performance,
			TAppCurrentHosts::<T>::iter_prefix(item.id).count() as u32,
			item.max_allowed_hosts,
			total_supply,
			buy_price,
			sell_price,
		)
	}

	/// Returned item fields:
	/// - CML Id
	/// - CML current performance
	/// - CML remaining performance
	/// - life remaining
	/// - Hosted tapp list
	pub fn list_candidate_miners(
		who: &T::AccountId,
	) -> Vec<(CmlId, Performance, Performance, T::BlockNumber, Vec<TAppId>)> {
		let current_block = frame_system::Pallet::<T>::block_number();
		let mining_cmls = T::CmlOperation::current_mining_cmls(None);

		mining_cmls
			.iter()
			.filter(|(cml_id, _)| match T::CmlOperation::cml_by_id(cml_id) {
				Ok(cml) => cml.owner().unwrap_or(&Default::default()).eq(who),
				Err(_) => false,
			})
			.map(|(cml_id, _)| {
				let (current_performance, _) =
					T::CmlOperation::miner_performance(*cml_id, &current_block);
				let hosted_performance = Self::cml_total_used_performance(*cml_id);
				let life_remain = match T::CmlOperation::cml_by_id(cml_id) {
					Ok(cml) => {
						let life_spends = current_block
							.saturating_sub(*cml.get_plant_at().unwrap_or(&Zero::zero()));
						cml.lifespan().saturating_sub(life_spends)
					}
					_ => Zero::zero(),
				};

				(
					*cml_id,
					current_performance.unwrap_or(0),
					current_performance
						.unwrap_or(0)
						.saturating_sub(hosted_performance),
					life_remain,
					CmlHostingTApps::<T>::get(cml_id),
				)
			})
			.collect()
	}

	/// `only_investing` if true will return only investing list, otherwise it will return
	/// 	investing and hosting reserved amount list
	/// Returned item fields:
	/// - Account
	/// - Staking amount
	pub fn tapp_staking_details(
		tapp_id: TAppId,
		only_investing: bool,
	) -> Vec<(T::AccountId, BalanceOf<T>)> {
		let mut details_list = BTreeMap::new();
		for (acc, id, amount) in AccountTable::<T>::iter() {
			if tapp_id != id {
				continue;
			}

			details_list.insert(acc, amount);
		}

		if !only_investing {
			TAppReservedBalance::<T>::iter_prefix(tapp_id).for_each(|(acc, list)| {
				let mut amount_sum: BalanceOf<T> = Zero::zero();
				list.into_iter()
					.for_each(|(amount, _)| amount_sum = amount_sum.saturating_add(amount));

				if amount_sum.is_zero() {
					return;
				}
				match details_list.get_mut(&acc) {
					Some(amount) => {
						*amount = amount.saturating_add(amount_sum);
					}
					None => {
						details_list.insert(acc, amount_sum);
					}
				}
			});
		}

		details_list.into_iter().collect()
	}

	/// Returned item fields:
	/// - CML Id
	/// - Owner account
	/// - life remaining
	/// - CML current performance
	/// - CML remaining performance
	/// - CML peak performance
	pub fn tapp_hosted_cmls(
		tapp_id: TAppId,
	) -> Vec<(
		CmlId,
		Option<T::AccountId>,
		T::BlockNumber,
		Option<Performance>,
		Option<Performance>,
		Performance,
	)> {
		let current_block = frame_system::Pallet::<T>::block_number();

		TAppCurrentHosts::<T>::iter_prefix(tapp_id)
			.map(|(cml_id, _)| {
				let (owner, life_remain) = match T::CmlOperation::cml_by_id(&cml_id) {
					Ok(cml) => {
						let life_spends = current_block
							.saturating_sub(*cml.get_plant_at().unwrap_or(&Zero::zero()));
						(
							cml.owner().cloned(),
							cml.lifespan().saturating_sub(life_spends),
						)
					}
					_ => (None, Zero::zero()),
				};
				let (current_performance, remaining_performance, peak_performance) =
					Self::cml_performance(cml_id);
				(
					cml_id,
					owner,
					life_remain,
					current_performance,
					remaining_performance,
					peak_performance,
				)
			})
			.collect()
	}

	/// Returned item fields:
	/// - CML Id
	/// - CML remaining performance
	/// - TApp Id
	/// - TApp Ticker
	/// - TApp Name
	/// - TApp Detail
	/// - TApp Link
	/// - Min performance request
	/// - Hosting stake token
	pub fn list_cml_hosting_tapps(
		cml_id: CmlId,
	) -> Vec<(
		CmlId,
		Option<Performance>,
		TAppId,
		Vec<u8>,
		Vec<u8>,
		Vec<u8>,
		Vec<u8>,
		Performance,
		BalanceOf<T>,
	)> {
		let (_, remaining_performance, _) = Self::cml_performance(cml_id);
		let owner = match T::CmlOperation::cml_by_id(&cml_id) {
			Ok(cml) => cml.owner().cloned().unwrap_or(Default::default()),
			Err(_) => Default::default(),
		};
		CmlHostingTApps::<T>::get(cml_id)
			.iter()
			.map(|tapp_id| {
				let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
				let host_performance = tapp_item.host_performance();

				let mut reserved_balance: BalanceOf<T> = Zero::zero();
				TAppReservedBalance::<T>::iter_prefix(tapp_id)
					.filter(|(account, _)| account.eq(&owner))
					.for_each(|(_, amount_list)| {
						amount_list.iter().for_each(|(balance, id)| {
							if cml_id == *id {
								reserved_balance = reserved_balance.saturating_add(balance.clone());
							}
						});
					});
				(
					cml_id,
					remaining_performance.clone(),
					*tapp_id,
					tapp_item.ticker,
					tapp_item.name,
					tapp_item.detail,
					tapp_item.link,
					host_performance,
					reserved_balance,
				)
			})
			.collect()
	}

	/// returned values:
	/// - current performance calculated by current block height
	/// - remaining performance
	/// - peak performance
	pub fn cml_performance(
		cml_id: CmlId,
	) -> (Option<Performance>, Option<Performance>, Performance) {
		let current_block = frame_system::Pallet::<T>::block_number();
		let (current_performance, peak_performance) =
			T::CmlOperation::miner_performance(cml_id, &current_block);
		let remaining_performance = current_performance.map(|performance| {
			let hosted_performance = Self::cml_total_used_performance(cml_id);
			performance.saturating_sub(hosted_performance)
		});
		(current_performance, remaining_performance, peak_performance)
	}

	/// Returned item fields:
	/// - Link url
	/// - Tapp id, if not created based on the link value will be none
	/// - Link description
	/// - Creator
	pub fn approved_links(
		allowed: bool,
	) -> Vec<(Vec<u8>, Option<u64>, Vec<u8>, Option<T::AccountId>)> {
		TAppApprovedLinks::<T>::iter()
			.filter(|(_, link_info)| !allowed || link_info.tapp_id.is_some())
			.map(|(link, link_info)| {
				(
					link,
					link_info.tapp_id,
					link_info.description,
					link_info.creator,
				)
			})
			.collect()
	}

	pub fn user_notification_count(
		account: T::AccountId,
		desired_start_height: T::BlockNumber,
	) -> u32 {
		let current_height = frame_system::Pallet::<T>::block_number();
		UserNotifications::<T>::get(account)
			.iter()
			.filter(|item| {
				current_height <= item.expired_height && desired_start_height <= item.start_height
			})
			.count() as u32
	}

	pub fn tapp_notifications_count(stop_height: T::BlockNumber) -> Vec<(TAppId, u32)> {
		let last_pay_height = NotificationsLastPayHeight::<T>::get();

		let mut notifications_count = BTreeMap::new();
		UserNotifications::<T>::iter().for_each(|(_, item_list)| {
			for item in item_list.iter() {
				if item.start_height > stop_height || item.start_height < last_pay_height {
					continue;
				}

				match notifications_count.get_mut(&item.tapp_id) {
					Some(count) => *count += 1,
					None => {
						notifications_count.insert(item.tapp_id, 1);
					}
				}
			}
		});

		notifications_count.into_iter().collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::*;
	use frame_support::assert_ok;

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

	#[test]
	fn tapp_notifications_fee_works() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let notification_account = 3;
			NotificationAccount::<Test>::set(Some(notification_account));

			let tapp_id = 1;
			let tapp_id2 = 2;
			let expired_height1 = 50;
			let expired_height2 = 80;

			let current_height1 = 10;
			frame_system::Pallet::<Test>::set_block_number(current_height1);
			assert_ok!(BondingCurve::push_notifications(
				Origin::signed(notification_account),
				vec![user1, user2],
				vec![expired_height1, expired_height2],
				tapp_id,
				b"test tsid".to_vec(),
			));

			let current_height2 = 30;
			frame_system::Pallet::<Test>::set_block_number(current_height2);
			assert_ok!(BondingCurve::push_notifications(
				Origin::signed(notification_account),
				vec![user1, user2],
				vec![expired_height1, expired_height2],
				tapp_id2,
				b"test tsid2".to_vec(),
			));

			assert_eq!(NotificationsLastPayHeight::<Test>::get(), 0);
			assert_eq!(
				BondingCurve::tapp_notifications_count(current_height1 - 1).len(),
				0
			);

			let result = BondingCurve::tapp_notifications_count(current_height1);
			assert_eq!(result.len(), 1);
			assert_eq!(result[0], (tapp_id, 2));

			let result = BondingCurve::tapp_notifications_count(current_height2);
			assert_eq!(result.len(), 2);
			assert_eq!(result[0], (tapp_id, 2));
			assert_eq!(result[1], (tapp_id2, 2));

			NotificationsLastPayHeight::<Test>::set(20);
			let result = BondingCurve::tapp_notifications_count(current_height2);
			assert_eq!(result.len(), 1);
			assert_eq!(result[0], (tapp_id2, 2));

			// expired notifications need to take into count either
			frame_system::Pallet::<Test>::set_block_number(60);
			let result = BondingCurve::tapp_notifications_count(80);
			assert_eq!(result.len(), 1);
			assert_eq!(result[0], (tapp_id2, 2));
		})
	}

	#[test]
	fn user_notification_count_works() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let notification_account = 3;
			NotificationAccount::<Test>::set(Some(notification_account));

			let tapp_id = 1;
			let tapp_id2 = 2;
			let expired_height1 = 50;
			let expired_height2 = 80;

			let current_height1 = 10;
			frame_system::Pallet::<Test>::set_block_number(current_height1);
			assert_ok!(BondingCurve::push_notifications(
				Origin::signed(notification_account),
				vec![user1, user2],
				vec![expired_height1, expired_height2],
				tapp_id,
				b"test tsid".to_vec(),
			));

			let current_height2 = 30;
			frame_system::Pallet::<Test>::set_block_number(current_height2);
			assert_ok!(BondingCurve::push_notifications(
				Origin::signed(notification_account),
				vec![user1, user2],
				vec![expired_height1, expired_height2],
				tapp_id2,
				b"test tsid2".to_vec(),
			));

			assert_eq!(BondingCurve::user_notification_count(user1, 0), 2);
			assert_eq!(BondingCurve::user_notification_count(user1, 20), 1);
			assert_eq!(BondingCurve::user_notification_count(user2, 0), 2);
			assert_eq!(BondingCurve::user_notification_count(user2, 20), 1);

			let current_height2 = 60;
			frame_system::Pallet::<Test>::set_block_number(current_height2);

			assert_eq!(BondingCurve::user_notification_count(user1, 0), 0);
			assert_eq!(BondingCurve::user_notification_count(user1, 20), 0);
			assert_eq!(BondingCurve::user_notification_count(user2, 0), 2);
			assert_eq!(BondingCurve::user_notification_count(user2, 20), 1);
			assert_eq!(BondingCurve::user_notification_count(user2, 40), 0);
		})
	}

	#[test]
	fn query_price_works() {
		new_test_ext().execute_with(|| {
			EnableUserCreateTApp::<Test>::set(true);
			<Test as Config>::Currency::make_free_balance_be(&1, DOLLARS * DOLLARS);

			let npc = NPCAccount::<Test>::get();
			let link = b"https://teaproject.org".to_vec();
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				link.clone(),
				"test description".into(),
				None,
			));

			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(1),
				b"test".to_vec(),
				b"test".to_vec(),
				DOLLARS * 10_000,
				vec![],
				link,
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			));
			let (buy_price, sell_price) = BondingCurve::query_price(1);
			assert_eq!(buy_price, 100000000000000);
			assert_eq!(sell_price, 70000000000000);

			let link2 = b"https://tearust.org".to_vec();
			assert_ok!(BondingCurve::register_tapp_link(
				Origin::signed(npc),
				link2.clone(),
				"test description2".into(),
				None,
			));
			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(1),
				b"test2".to_vec(),
				b"test2".to_vec(),
				DOLLARS * 1_000_000,
				vec![],
				link2,
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			));
			let (buy_price, sell_price) = BondingCurve::query_price(2);
			assert_eq!(buy_price, 1000000000000000);
			assert_eq!(sell_price, 700000000000000);
		})
	}
}
