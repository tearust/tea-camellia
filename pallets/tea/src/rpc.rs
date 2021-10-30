use super::*;

impl<T: tea::Config> tea::Pallet<T> {
	pub fn is_ra_validator(
		tea_id: &[u8; 32],
		target_tea_id: &[u8; 32],
		block_number: &T::BlockNumber,
	) -> bool {
		Self::is_validator(tea_id, target_tea_id, block_number)
	}
}
