use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	// pub(super) fn get_random_life() -> T::BlockNumber {
	// 	10_000_000.into()
	// }

	// pub(super) fn get_random_mining_rate() -> u8 {
	// 	10 as u8
	// }

	// pub(super) fn get_next_id() -> T::AssetId {
	// 	let cid = LastAssetId::<T>::get();
	// 	let id = cid.clone;
	// 	LastAssetId::<T>::mutate(|id| *id += One::one());

	// 	cid
	// }

	pub(crate) fn get_dai(who: &T::AccountId) -> Dai {
    match DaiStore::<T>::get(&who) {
			Some(n) => n,
			None => 0 as Dai,
		}
  }

  pub fn set_dai(
    who: &T::AccountId,
    amount: Dai
  ) {
    DaiStore::<T>::mutate(&who, |n| *n = Some(amount));
	}
}
