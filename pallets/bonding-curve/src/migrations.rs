use super::*;

pub(crate) mod v1 {
	use super::*;

	#[cfg(feature = "try-runtime")]
	pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		Ok(())
	}

	pub(crate) fn migrate<T: Config>() -> Weight {
		let mut reads_writes = 0;

		TAppBondingCurve::<T>::translate::<
			super::v0::TAppItemV0<T::AccountId, BalanceOf<T>, T::BlockNumber>,
			_,
		>(|_key, old_tapp| {
			reads_writes += 1;
			let v: Option<TAppItem<T::AccountId, BalanceOf<T>, T::BlockNumber>> =
				Some(old_tapp.into());
			v
		});

		T::DbWeight::get().reads_writes(reads_writes, reads_writes)
	}

	#[cfg(feature = "try-runtime")]
	pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
		Ok(())
	}
}

pub(crate) mod v0 {
	use super::*;

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct TAppItemV0<AccountId, Balance, BlockNumber> {
		pub id: TAppId,
		pub name: Vec<u8>,
		pub ticker: Vec<u8>,
		pub owner: AccountId,
		pub buy_curve: CurveType,
		pub sell_curve: CurveType,
		pub detail: Vec<u8>,
		pub link: Vec<u8>,
		pub max_allowed_hosts: u32,
		pub current_cost: Balance,
		pub status: TAppStatus<BlockNumber>,
		pub tapp_type: TAppType,
		pub billing_mode: BillingMode<Balance>,
	}

	impl<AccountId, Balance, BlockNumber> Default for TAppItemV0<AccountId, Balance, BlockNumber>
	where
		AccountId: Default,
		Balance: AtLeast32BitUnsigned + Default,
	{
		fn default() -> Self {
			TAppItemV0 {
				id: 0,
				name: vec![],
				ticker: vec![],
				owner: Default::default(),
				buy_curve: CurveType::UnsignedLinear,
				sell_curve: CurveType::UnsignedLinear,
				detail: vec![],
				link: vec![],
				max_allowed_hosts: Default::default(),
				current_cost: Default::default(),
				status: TAppStatus::Pending,
				tapp_type: TAppType::Twitter,
				billing_mode: BillingMode::FixedHostingToken(Default::default()),
			}
		}
	}

	impl<AccountId, Balance, BlockNumber> Into<TAppItem<AccountId, Balance, BlockNumber>>
		for TAppItemV0<AccountId, Balance, BlockNumber>
	where
		AccountId: Default,
		Balance: AtLeast32BitUnsigned + Default,
	{
		fn into(self) -> TAppItem<AccountId, Balance, BlockNumber> {
			let to_theta = |t| match t {
				CurveType::UnsignedSquareRoot_7 => 7u32,
				CurveType::UnsignedSquareRoot_10 => 10u32,
				_ => 10u32,
			};

			TAppItem {
				id: self.id,
				name: self.name,
				ticker: self.ticker,
				owner: self.owner,
				detail: self.detail,
				link: self.link,
				max_allowed_hosts: self.max_allowed_hosts,
				current_cost: self.current_cost,
				status: self.status,
				tapp_type: self.tapp_type,
				billing_mode: self.billing_mode,
				buy_curve_theta: to_theta(self.buy_curve),
				sell_curve_theta: to_theta(self.sell_curve),
			}
		}
	}
}
