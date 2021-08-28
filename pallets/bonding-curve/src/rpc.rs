use super::*;
use pallet_cml::{SeedProperties, TreeProperties};

impl<T: bonding_curve::Config> bonding_curve::Pallet<T> {
	pub fn query_price(tapp_id: TAppId) -> (BalanceOf<T>, BalanceOf<T>) {
		let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let buy_price = match tapp_item.buy_curve {
			CurveType::UnsignedLinear => T::LinearCurve::buy_price(total_supply),
			CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::buy_price(total_supply),
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::buy_price(total_supply),
		};
		let sell_price = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::buy_price(total_supply),
			CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::buy_price(total_supply),
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::buy_price(total_supply),
		};
		(buy_price, sell_price)
	}

	pub fn estimate_required_tea_when_buy(
		tapp_id: Option<TAppId>,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		match Self::calculate_buy_amount(tapp_id, tapp_amount) {
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
			Ok(balance) => balance,
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
	/// - current hosts (return zero if is none)
	/// - max hosts (return zero if is none)
	pub fn list_tapps() -> Vec<(
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
		u32,
		u32,
	)> {
		TAppBondingCurve::<T>::iter()
			.map(|(id, item)| {
				let (buy_price, sell_price) = Self::query_price(id);
				let total_supply = TotalSupplyTable::<T>::get(id);

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
					item.host_performance.unwrap_or_default(),
					TAppCurrentHosts::<T>::iter_prefix(item.id).count() as u32,
					item.max_allowed_hosts.unwrap_or_default(),
				)
			})
			.collect()
	}

	/// Returned item fields:
	/// - TApp Name
	/// - TApp Id
	/// - TApp Ticker
	/// - User holding tokens
	/// - Token sell price
	/// - Owner
	/// - Detail
	/// - Link
	/// - Host performance requirement (return zero if is none)
	/// - current hosts (return zero if is none)
	/// - max hosts (return zero if is none)
	pub fn list_user_assets(
		who: &T::AccountId,
	) -> Vec<(
		Vec<u8>,
		TAppId,
		Vec<u8>,
		BalanceOf<T>,
		BalanceOf<T>,
		T::AccountId,
		Vec<u8>,
		Vec<u8>,
		Performance,
		u32,
		u32,
	)> {
		AccountTable::<T>::iter_prefix(who)
			.map(|(id, amount)| {
				let (_, sell_price) = Self::query_price(id);
				let item = TAppBondingCurve::<T>::get(id);

				(
					item.name,
					id,
					item.ticker,
					amount,
					sell_price,
					item.owner,
					item.detail,
					item.link,
					item.host_performance.unwrap_or_default(),
					TAppCurrentHosts::<T>::iter_prefix(item.id).count() as u32,
					item.max_allowed_hosts.unwrap_or_default(),
				)
			})
			.collect()
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
			.filter(|cml_id| match T::CmlOperation::cml_by_id(cml_id) {
				Ok(cml) => cml.owner().unwrap_or(&Default::default()).eq(who),
				Err(_) => false,
			})
			.map(|cml_id| {
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

			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(1),
				b"test".to_vec(),
				b"test".to_vec(),
				DOLLARS * 10_000,
				vec![],
				vec![],
				None,
				None,
			));
			let (buy_price, sell_price) = BondingCurve::query_price(1);
			assert_eq!(buy_price, 100000000000000);
			assert_eq!(sell_price, 70000000000000);

			assert_ok!(BondingCurve::create_new_tapp(
				Origin::signed(1),
				b"test2".to_vec(),
				b"test2".to_vec(),
				DOLLARS * 1_000_000,
				vec![],
				vec![],
				None,
				None,
			));
			let (buy_price, sell_price) = BondingCurve::query_price(2);
			assert_eq!(buy_price, 1000000000000000);
			assert_eq!(sell_price, 700000000000000);
		})
	}
}
