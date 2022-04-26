use super::*;

impl<T: tea::Config> tea::Pallet<T> {
	pub fn next_id() -> IssuerId {
		LastIssuerId::<T>::mutate(|id| {
			if *id < u64::MAX {
				*id += 1;
			} else {
				*id = 1;
			}

			*id
		})
	}
}
