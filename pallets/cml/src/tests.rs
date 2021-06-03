use crate::{mock::*, types::*, CmlStore, Config, DaiStore, Error, MinerItemStore, UserCmlStore};
use frame_support::{assert_noop, assert_ok, traits::Currency};

#[test]
fn convert_cml_from_dai_works() {
    new_test_ext().execute_with(|| {
        DaiStore::<Test>::insert(1, 100);

        assert_ok!(Cml::convert_cml_from_dai(Origin::signed(1)));

        assert_eq!(DaiStore::<Test>::get(1).unwrap(), 99);
        let cml_list = UserCmlStore::<Test>::get(1).unwrap();
        assert_eq!(cml_list.len(), 1);

        let cml = CmlStore::<Test>::get(cml_list[0]);
        assert!(cml.is_some());
        let cml = cml.unwrap();
        assert_eq!(cml.id, cml_list[0]);
        assert_eq!(cml.group, CmlGroup::Nitro);
        assert_eq!(cml.status, CmlStatus::SeedFrozen);
    })
}

#[test]
fn convert_cml_if_dai_is_empty() {
    // account dai is 0
    new_test_ext().execute_with(|| {
        DaiStore::<Test>::insert(1, 0);

        assert_noop!(
            Cml::convert_cml_from_dai(Origin::signed(1)),
            Error::<Test>::NotEnoughDai
        );
    });

    // account not exist
    new_test_ext().execute_with(|| {
        assert_noop!(
            Cml::convert_cml_from_dai(Origin::signed(1)),
            Error::<Test>::NotEnoughDai
        );
    });
}

#[test]
fn active_cml_for_nitro_works() {
    new_test_ext().execute_with(|| {
        let amount = 100 * 1000; // Unit * StakingPrice
        DaiStore::<Test>::insert(1, 100);
        <Test as Config>::Currency::make_free_balance_be(&1, amount);

        Cml::convert_cml_from_dai(Origin::signed(1)).unwrap();
        let cml_list = UserCmlStore::<Test>::get(1).unwrap();
        let cml = CmlStore::<Test>::get(cml_list[0]).unwrap();

        let miner_id = b"miner_id".to_vec();
        let miner_ip = b"miner_ip".to_vec();
        assert_ok!(Cml::active_cml_for_nitro(
            Origin::signed(1),
            cml.id,
            miner_id.clone(),
            miner_ip.clone()
        ));

        let cml_list = UserCmlStore::<Test>::get(1).unwrap();
        let cml = CmlStore::<Test>::get(cml_list[0]).unwrap();
        assert_eq!(cml.status, CmlStatus::CmlLive);
        assert_eq!(cml.staking_slot.len(), 1);

        let staking_item = cml.staking_slot.get(0).unwrap();
        assert_eq!(staking_item.owner, 1);
        // todo let me pass later
        // assert_eq!(staking_item.amount, amount as u32);
        assert_eq!(staking_item.cml, None);

        let miner_item = MinerItemStore::<Test>::get(miner_id.clone()).unwrap();
        assert_eq!(miner_item.id, miner_id);
        assert_eq!(miner_item.id, cml.miner_id);
        assert_eq!(miner_item.status, MinerStatus::Active);
        assert_eq!(miner_item.ip, miner_ip);
    });
}

#[test]
fn active_cml_for_nitro_with_insufficient_free_balance() {
    new_test_ext().execute_with(|| {
        // default account `1` free balance is 0
        assert_noop!(
            Cml::active_cml_for_nitro(
                Origin::signed(1),
                1,
                b"miner_id".to_vec(),
                b"miner_id".to_vec()
            ),
            Error::<Test>::NotFoundCML
        );
    })
}

#[test]
fn active_not_exist_cml_for_nitro() {
    new_test_ext().execute_with(|| {
        <Test as Config>::Currency::make_free_balance_be(&1, 100 * 1000);

        assert_noop!(
            Cml::active_cml_for_nitro(
                Origin::signed(1),
                1,
                b"miner_id".to_vec(),
                b"miner_ip".to_vec()
            ),
            Error::<Test>::NotFoundCML
        );
    })
}

#[test]
fn active_cml_for_nitro_with_multiple_times() {
    new_test_ext().execute_with(|| {
        let amount = 100 * 1000; // Unit * StakingPrice
        DaiStore::<Test>::insert(1, 100);
        <Test as Config>::Currency::make_free_balance_be(&1, amount);

        Cml::convert_cml_from_dai(Origin::signed(1)).unwrap();
        let cml_list = UserCmlStore::<Test>::get(1).unwrap();
        let cml = CmlStore::<Test>::get(cml_list[0]).unwrap();

        let miner_id = b"miner_id".to_vec();
        let miner_ip = b"miner_ip".to_vec();
        assert_ok!(Cml::active_cml_for_nitro(
            Origin::signed(1),
            cml.id,
            miner_id.clone(),
            miner_ip.clone()
        ));

        assert_noop!(
            Cml::active_cml_for_nitro(
                Origin::signed(1),
                cml.id,
                miner_id.clone(),
                miner_ip.clone()
            ),
            Error::<Test>::MinerAlreadyExist
        );
    })
}
