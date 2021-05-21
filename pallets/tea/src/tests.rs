use crate::{mock::*, Error, Nodes};
use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;

#[test]
fn test_add_new_node() {
    new_test_ext().execute_with(|| {
        let public: [u8; 32] =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        assert_ok!(TeaModule::add_new_node(Origin::signed(1), public));
        let target_node = Nodes::<Test>::get(&public).unwrap();
        assert!(target_node.is_some());
        let target_node = target_node.unwrap();
        assert_eq!(
            target_node.create_time,
            frame_system::Pallet::<Test>::block_number()
        );
    })
}

#[test]
fn test_add_new_node_already_exist() {
    new_test_ext().execute_with(|| {
        let public: [u8; 32] =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        let _ = TeaModule::add_new_node(Origin::signed(1), public);

        assert_noop!(
            TeaModule::add_new_node(Origin::signed(1), public),
            Error::<Test>::NodeAlreadyExist
        );
    })
}
