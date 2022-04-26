use frame_support::assert_ok;

use crate::{
	mock::{new_test_ext, Cml, Origin, Test},
	CmlId, CmlStore, LastCmlId, NPCAccount, UserCmlStore,
};

#[test]
fn generate_cml_works() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);
		let npc = 1;
		NPCAccount::<Test>::set(npc);

		let a_count = 4;
		let b_count = 5;
		assert_ok!(Cml::generate_cml(Origin::signed(npc), a_count, b_count));

		assert_eq!(CmlStore::<Test>::iter().count() as u32, a_count + b_count);
		assert_eq!(LastCmlId::<Test>::get() as u32, a_count + b_count);

		for i in 0..(a_count + b_count) {
			assert!(CmlStore::<Test>::contains_key(i as CmlId));
			let cml = CmlStore::<Test>::get(i as CmlId);
			assert_eq!(cml.owner(), &NPCAccount::<Test>::get());
			assert!(UserCmlStore::<Test>::contains_key(
				NPCAccount::<Test>::get(),
				i as CmlId
			));
		}

		assert_ok!(Cml::generate_cml(Origin::signed(npc), 3, 0));
		assert_eq!(
			CmlStore::<Test>::iter().count() as u32,
			a_count + b_count + 3
		);
		assert_eq!(LastCmlId::<Test>::get() as u32, a_count + b_count + 3);
	})
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);
		let npc = 1;
		NPCAccount::<Test>::set(npc);

		assert_ok!(Cml::generate_cml(Origin::signed(npc), 1, 1));
		assert_eq!(CmlStore::<Test>::get(0).owner(), &NPCAccount::<Test>::get());
		assert!(UserCmlStore::<Test>::contains_key(
			NPCAccount::<Test>::get(),
			0
		));

		assert_eq!(CmlStore::<Test>::get(1).owner(), &NPCAccount::<Test>::get());
		assert!(UserCmlStore::<Test>::contains_key(
			NPCAccount::<Test>::get(),
			1
		));

		let user1 = 2;
		let user2 = 3;
		assert_ok!(Cml::transfer(Origin::signed(npc), 0, user1));
		assert_ok!(Cml::transfer(Origin::signed(npc), 1, user2));
		assert_eq!(CmlStore::<Test>::get(0).owner(), &user1);
		assert!(UserCmlStore::<Test>::contains_key(user1, 0));
		assert_eq!(CmlStore::<Test>::get(1).owner(), &user2);
		assert!(UserCmlStore::<Test>::contains_key(user2, 1));
	})
}
