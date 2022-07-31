use crate::{
	mock::{new_test_ext, Cml, Origin, Test},
	CmlId, CmlStore, LastCmlId, NPCAccount, UserCmlStore,
};
use frame_support::assert_ok;

#[test]
fn generate_cml_works() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);
		let npc = 1;
		NPCAccount::<Test>::set(Some(npc));

		let b_count = 5;
		assert_ok!(Cml::generate_cml(Origin::signed(npc), b_count));

		assert_eq!(CmlStore::<Test>::iter().count() as u32, b_count);
		assert_eq!(LastCmlId::<Test>::get() as u32, b_count);

		for i in 0..b_count {
			assert!(CmlStore::<Test>::contains_key(i as CmlId));
			let cml = CmlStore::<Test>::get(i as CmlId).unwrap();
			assert_eq!(cml.owner(), &NPCAccount::<Test>::get().unwrap());
			assert!(UserCmlStore::<Test>::contains_key(
				NPCAccount::<Test>::get().unwrap(),
				i as CmlId
			));
		}

		assert_ok!(Cml::generate_cml(Origin::signed(npc), 3));
		assert_eq!(CmlStore::<Test>::iter().count() as u32, b_count + 3);
		assert_eq!(LastCmlId::<Test>::get() as u32, b_count + 3);
	})
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);
		let npc = 1;
		NPCAccount::<Test>::set(Some(npc));

		assert_ok!(Cml::generate_cml(Origin::signed(npc), 1));
		assert_eq!(
			CmlStore::<Test>::get(0).unwrap().owner(),
			&NPCAccount::<Test>::get().unwrap()
		);
		assert!(UserCmlStore::<Test>::contains_key(
			NPCAccount::<Test>::get().unwrap(),
			0
		));

		let user1 = 2;
		assert_ok!(Cml::transfer(Origin::signed(npc), 0, user1));
		assert_eq!(CmlStore::<Test>::get(0).unwrap().owner(), &user1);
		assert!(UserCmlStore::<Test>::contains_key(user1, 0));
		assert!(!UserCmlStore::<Test>::contains_key(npc, 0));
	})
}
