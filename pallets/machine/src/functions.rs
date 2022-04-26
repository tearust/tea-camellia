use super::*;

impl<T: tea::Config> tea::Pallet<T> {
	pub(crate) fn verify_ed25519_signature(
		pubkey: &TeaPubKey,
		content: &[u8],
		signature: &Signature,
	) -> DispatchResult {
		let ed25519_pubkey = ed25519::Public(pubkey.clone());
		ensure!(signature.len() == 64, Error::<T>::InvalidSignatureLength);
		let ed25519_sig = ed25519::Signature::from_slice(&signature[..]);
		ensure!(
			ed25519_sig.verify(content, &ed25519_pubkey),
			Error::<T>::InvalidSignature
		);
		Ok(())
	}

	pub(crate) fn versions_hash(versions: &Vec<VersionItem>) -> H256 {
		versions.using_encoded(blake2_256).into()
	}
}
