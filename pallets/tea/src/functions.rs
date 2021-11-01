use super::*;

impl<T: tea::Config> TeaOperation for tea::Pallet<T> {
	type AccountId = T::AccountId;

	fn add_new_node(machine_id: [u8; 32], sender: &Self::AccountId) {
		let current_block_number = frame_system::Pallet::<T>::block_number();

		let mut new_node = Node::default();
		new_node.tea_id = machine_id.clone();
		new_node.create_time = current_block_number;
		new_node.update_time = current_block_number;
		Nodes::<T>::insert(machine_id, new_node.clone());

		Self::deposit_event(Event::NewNodeJoined(sender.clone(), new_node));
	}
}

impl<T: tea::Config> tea::Pallet<T> {
	pub(crate) fn should_pay_report_reward(n: &T::BlockNumber) -> bool {
		// offset with `ReportRawardDuration` - 3 to void overlapping with staking period
		*n % T::ReportRawardDuration::get() == T::ReportRawardDuration::get() - 3u32.into()
	}

	pub(crate) fn check_tea_id_belongs(
		sender: &T::AccountId,
		tea_id: &TeaPubKey,
	) -> DispatchResult {
		if !BuiltinNodes::<T>::contains_key(tea_id) {
			ensure!(
				T::CmlOperation::check_miner(*tea_id, sender),
				Error::<T>::InvalidTeaIdOwner
			);
		} else {
			ensure!(
				BuiltinMiners::<T>::contains_key(sender),
				Error::<T>::InvalidBuiltinMiner
			);
		}
		Ok(())
	}

	pub(crate) fn pop_existing_node(tea_id: &TeaPubKey) -> Node<T::BlockNumber> {
		let old_node = Nodes::<T>::get(tea_id);
		EphemeralIds::<T>::remove(&old_node.ephemeral_id);
		PeerIds::<T>::remove(&old_node.peer_id);
		old_node
	}

	pub(crate) fn is_builtin_node(tea_id: &TeaPubKey) -> bool {
		BuiltinNodes::<T>::contains_key(tea_id)
	}

	pub(crate) fn initial_node_status(tea_id: &TeaPubKey) -> NodeStatus {
		match Self::is_builtin_node(tea_id) {
			true => NodeStatus::Active,
			false => NodeStatus::Pending,
		}
	}

	pub(crate) fn update_node_status(
		tea_id: &TeaPubKey,
		target_tea_id: &TeaPubKey,
		is_pass: bool,
	) -> Option<NodeStatus> {
		Nodes::<T>::mutate(target_tea_id, |target_node| {
			target_node.ra_nodes.push((tea_id.clone(), is_pass));
			if target_node.status == NodeStatus::Active {
				None
			} else {
				let group_id =
					Self::group_id(target_tea_id, ValidatorGroupsCount::<T>::iter().count());

				if target_node
					.ra_nodes
					.iter()
					.filter(|(_, pass)| *pass)
					.count() as u32 >= (ValidatorGroupsCount::<T>::get(group_id) + 1) / 2
				{
					// set RA node status to active if more than have validators agreed
					target_node.status = NodeStatus::Active;
					Some(NodeStatus::Active)
				} else {
					None
				}
			}
		})
	}

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

	pub(crate) fn try_suspend_node(offline_tea_id: &TeaPubKey) {
		if OfflineEvidences::<T>::get(offline_tea_id).len()
			< T::OfflineEffectThreshold::get() as usize
		{
			return;
		}

		T::CmlOperation::suspend_mining(*offline_tea_id);
		OfflineEvidences::<T>::remove(offline_tea_id);
	}

	pub(crate) fn pay_report_reward() {
		ReportEvidences::<T>::iter().for_each(|(_, ev)| {
			if let Some(cml) = T::CmlOperation::cml_by_machine_id(&ev.reporter) {
				if let Some(owner) = cml.owner() {
					T::CurrencyOperations::deposit_creating(owner, T::ReportRawardAmount::get());
				}
			}
		});
		ReportEvidences::<T>::remove_all(None);
	}
}

#[cfg(test)]
mod tests {
	use crate::{mock::*, types::*, Error, Nodes};
	use frame_support::{assert_noop, assert_ok};

	#[test]
	fn verify_ed25519_signature_works() {
		use ed25519_dalek::ed25519::signature::Signature;
		use ed25519_dalek::{Keypair, Signer};
		use rand::rngs::OsRng;

		new_test_ext().execute_with(|| {
			let tea_id = [3u8; 32];
			let mut csprng = OsRng {};
			let kp = Keypair::generate(&mut csprng);
			let signature = kp.sign(&tea_id);

			assert!(kp.verify(&tea_id, &signature).is_ok());
			assert_ok!(Tea::verify_ed25519_signature(
				&kp.public.as_bytes(),
				&tea_id,
				&signature.as_bytes().to_vec(),
			));

			assert_noop!(
				Tea::verify_ed25519_signature(
					&kp.public.as_bytes(),
					&tea_id,
					&vec![0u8; 33], // wrong signature length
				),
				Error::<Test>::InvalidSignatureLength
			);

			let wrong_message = [2u8; 32];
			assert!(kp.verify(&wrong_message, &signature).is_err());
			assert_noop!(
				Tea::verify_ed25519_signature(
					&kp.public.as_bytes(),
					&wrong_message,
					&signature.as_bytes().to_vec(),
				),
				Error::<Test>::InvalidSignature
			);
		})
	}

	#[test]
	fn update_runtime_status_works() {
		new_test_ext().execute_with(|| {
			let initial_height = 10;
			let threshold_height = RUNTIME_ACTIVITY_THRESHOLD as u64;

			let tea_id1: TeaPubKey = [1; 32];
			let mut node1 = Node::default();
			node1.update_time = initial_height;
			node1.status = NodeStatus::Active;
			Nodes::<Test>::insert(&tea_id1, node1);

			let tea_id2: TeaPubKey = [2; 32];
			let mut node2 = Node::default();
			node2.update_time = initial_height + 1;
			node2.status = NodeStatus::Active;
			Nodes::<Test>::insert(&tea_id2, node2);

			Tea::update_runtime_status(initial_height + 2);
			assert_eq!(Nodes::<Test>::get(&tea_id1).status, NodeStatus::Active);
			assert_eq!(Nodes::<Test>::get(&tea_id2).status, NodeStatus::Active);

			Tea::update_runtime_status(initial_height + threshold_height);
			assert_eq!(Nodes::<Test>::get(&tea_id1).status, NodeStatus::Active);
			assert_eq!(Nodes::<Test>::get(&tea_id2).status, NodeStatus::Active);

			Tea::update_runtime_status(initial_height + threshold_height + 1);
			assert_eq!(Nodes::<Test>::get(&tea_id1).status, NodeStatus::Inactive);
			assert_eq!(Nodes::<Test>::get(&tea_id2).status, NodeStatus::Active);

			Tea::update_runtime_status(initial_height + threshold_height + 2);
			assert_eq!(Nodes::<Test>::get(&tea_id1).status, NodeStatus::Inactive);
			assert_eq!(Nodes::<Test>::get(&tea_id2).status, NodeStatus::Inactive);
		});
	}
}
