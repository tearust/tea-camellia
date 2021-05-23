use super::*;

impl<T: Config> Pallet<T> {
	pub(super) fn get_random_life() -> T::BlockNumber {
		10_000_000.into()
	}

	pub(super) fn get_random_mining_rate() -> u8 {
		10 as u8
	}

	pub(super) fn get_next_id() -> T::AssetId {
		let cid = LastAssetId::<T>::get();
		let id = cid.clone;
		LastAssetId::<T>::mutate(|id| *id += One::one());

		cid
	}

	pub fn get_dai(who: &T::AccountId) -> T::Dai {
    let n = <DaiStore<T>>::get(&who);
    n
  }

  fn set_dai(
    who: &T::AccountId,
    amount: T::Dai
  ) {
    <DaiStore<T>>::mutate(&who, |n| *n = amount);
	}
}
