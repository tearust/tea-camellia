use super::*;

impl<T: bonding_curve::Config> bonding_curve::Pallet<T> {
	pub(crate) fn tsid_hash(tsid: &[u8]) -> H256 {
		tsid.using_encoded(blake2_256).into()
	}
}
