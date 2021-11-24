use super::*;

impl<T: tea::Config> tea::Pallet<T> {
	pub fn is_ra_validator(
		tea_id: &[u8; 32],
		target_tea_id: &[u8; 32],
		block_number: &T::BlockNumber,
	) -> bool {
		Self::is_validator(tea_id, target_tea_id, block_number)
	}

	pub fn list_boot_nodes() -> Vec<[u8; 32]> {
		BuiltinNodes::<T>::iter().map(|(id, _)| id).collect()
	}

	pub fn list_allowed_pcrs() -> Vec<(H256, Vec<PcrValue>)> {
		AllowedPcrValues::<T>::iter()
			.map(|(hash, v)| (hash, v.slots))
			.collect()
	}

	pub fn list_allowed_versions() -> Vec<(H256, Vec<VersionItem>)> {
		AllowedVersions::<T>::iter()
			.map(|(hash, v)| (hash, v.versions))
			.collect()
	}

	pub fn list_version_expired_nodes() -> Vec<[u8; 32]> {
		VersionExpiredNodes::<T>::iter().map(|(id, _)| id).collect()
	}

	pub fn find_tea_id_by_peer_id(peer_id: &[u8]) -> Vec<[u8; 32]> {
		Nodes::<T>::iter()
			.filter(|(_, node)| node.peer_id.eq(peer_id))
			.map(|(id, _)| id)
			.collect()
	}
}
