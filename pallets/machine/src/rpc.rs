use super::*;

impl<T: tea::Config> tea::Pallet<T> {
	pub fn list_boot_nodes() -> Vec<[u8; 32]> {
		BuiltinNodes::<T>::iter().map(|(id, _)| id).collect()
	}

	pub fn list_tapp_store_startup_nodes() -> Vec<[u8; 32]> {
		TappStoreStartupNodes::<T>::get()
	}
}
