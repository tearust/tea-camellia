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
		let mining_cmls = T::CmlOperation::current_mining_cmls();

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
	pub fn approved_links() -> Vec<(Vec<u8>, Option<u64>, Vec<u8>, Option<T::AccountId>)> {
		TAppApprovedLinks::<T>::iter()
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
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::*;
	use frame_support::assert_ok;

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

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
