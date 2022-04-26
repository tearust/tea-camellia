use super::*;

impl<T: tea::Config> tea::Pallet<T> {
	pub fn list_boot_nodes() -> Vec<[u8; 32]> {
		vec![]
	}

	pub fn list_tapp_store_startup_nodes() -> Vec<[u8; 32]> {
		vec![]
	}
}
