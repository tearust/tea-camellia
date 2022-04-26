use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub fn user_cml_list(who: &T::AccountId) -> Vec<u64> {
		UserCmlStore::<T>::iter_prefix(who)
			.map(|(id, _)| id)
			.collect()
	}
}
