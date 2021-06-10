#![cfg(test)]

use super::mock::{
    assert_events, balances, event_exists, expect_event, new_test_ext, Balances, Bridge, Call,
    ChainBridge, Event, HashId, NativeTokenId, Origin, ProposalLifetime, ENDOWED_BALANCE,
    RELAYER_A, RELAYER_B, RELAYER_C,
};
use super::*;
use frame_support::dispatch::DispatchError;
use frame_support::{assert_noop, assert_ok};

use codec::Encode;
use sp_core::{blake2_256, H256};

const TEST_THRESHOLD: u32 = 2;

fn make_remark_proposal(hash: H256) -> Call {
    let resource_id = HashId::get();
    Call::Bridge(crate::Call::remark(hash, resource_id))
}

fn make_transfer_proposal(to: u64, amount: u64) -> Call {
    let resource_id = HashId::get();
    Call::Bridge(crate::Call::transfer(to, amount.into(), resource_id))
}

#[test]
fn transfer_hash() {
    new_test_ext().execute_with(|| {
        let dest_chain = 0;
        let resource_id = HashId::get();
        let hash: H256 = "ABC".using_encoded(blake2_256).into();

        assert_ok!(ChainBridge::set_threshold(Origin::root(), TEST_THRESHOLD,));

        assert_ok!(ChainBridge::whitelist_chain(
            Origin::root(),
            dest_chain.clone()
        ));
        assert_ok!(Bridge::transfer_hash(
            Origin::signed(1),
            hash.clone(),
            dest_chain,
        ));

        expect_event(chainbridge::RawEvent::GenericTransfer(
            dest_chain,
            1,
            resource_id,
            hash.as_ref().to_vec(),
        ));
    })
}

#[test]
fn transfer_native() {
    new_test_ext().execute_with(|| {
        let dest_chain = 0;
        let resource_id = NativeTokenId::get();
        let amount: u64 = 100;
        let recipient = vec![99];

        assert_ok!(ChainBridge::whitelist_chain(
            Origin::root(),
            dest_chain.clone()
        ));
        assert_ok!(Bridge::transfer_native(
            Origin::signed(RELAYER_A),
            amount.clone(),
            recipient.clone(),
            dest_chain,
        ));

        expect_event(chainbridge::RawEvent::FungibleTransfer(
            dest_chain,
            1,
            resource_id,
            amount.into(),
            recipient,
        ));
    })
}

#[test]
fn execute_remark() {
    new_test_ext().execute_with(|| {
        let hash: H256 = "ABC".using_encoded(blake2_256).into();
        let proposal = make_remark_proposal(hash.clone());
        let prop_id = 1;
        let src_id = 1;
        let r_id = chainbridge::derive_resource_id(src_id, b"hash");
        let resource = b"Bridge.remark".to_vec();

        assert_ok!(ChainBridge::set_threshold(Origin::root(), TEST_THRESHOLD,));
        assert_ok!(ChainBridge::add_relayer(Origin::root(), RELAYER_A));
        assert_ok!(ChainBridge::add_relayer(Origin::root(), RELAYER_B));
        assert_ok!(ChainBridge::whitelist_chain(Origin::root(), src_id));
        assert_ok!(ChainBridge::set_resource(Origin::root(), r_id, resource));

        assert_ok!(ChainBridge::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        assert_ok!(ChainBridge::acknowledge_proposal(
            Origin::signed(RELAYER_B),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));

        event_exists(RawEvent::Remark(hash));
    })
}

#[test]
fn execute_remark_bad_origin() {
    new_test_ext().execute_with(|| {
        let hash: H256 = "ABC".using_encoded(blake2_256).into();
        let resource_id = HashId::get();
        assert_ok!(Bridge::remark(
            Origin::signed(ChainBridge::account_id()),
            hash,
            resource_id
        ));
        // Don't allow any signed origin except from bridge addr
        assert_noop!(
            Bridge::remark(Origin::signed(RELAYER_A), hash, resource_id),
            DispatchError::BadOrigin
        );
        // Don't allow root calls
        assert_noop!(
            Bridge::remark(Origin::root(), hash, resource_id),
            DispatchError::BadOrigin
        );
    })
}

#[test]
fn transfer() {
    new_test_ext().execute_with(|| {
        // Check inital state
        let bridge_id: u64 = ChainBridge::account_id();
        let resource_id = HashId::get();
        assert_eq!(Balances::free_balance(&bridge_id), ENDOWED_BALANCE);
        // Transfer and check result
        assert_ok!(Bridge::transfer(
            Origin::signed(ChainBridge::account_id()),
            RELAYER_A,
            10,
            resource_id,
        ));
        assert_eq!(Balances::free_balance(&bridge_id), ENDOWED_BALANCE - 10);
        assert_eq!(Balances::free_balance(RELAYER_A), ENDOWED_BALANCE + 10);

        assert_events(vec![Event::balances(balances::Event::Transfer(
            ChainBridge::account_id(),
            RELAYER_A,
            10,
        ))]);
    })
}

#[test]
fn create_sucessful_transfer_proposal() {
    new_test_ext().execute_with(|| {
        let prop_id = 1;
        let src_id = 1;
        let r_id = chainbridge::derive_resource_id(src_id, b"transfer");
        let resource = b"Bridge.transfer".to_vec();
        let proposal = make_transfer_proposal(RELAYER_A, 10);

        assert_ok!(ChainBridge::set_threshold(Origin::root(), TEST_THRESHOLD,));
        assert_ok!(ChainBridge::add_relayer(Origin::root(), RELAYER_A));
        assert_ok!(ChainBridge::add_relayer(Origin::root(), RELAYER_B));
        assert_ok!(ChainBridge::add_relayer(Origin::root(), RELAYER_C));
        assert_ok!(ChainBridge::whitelist_chain(Origin::root(), src_id));
        assert_ok!(ChainBridge::set_resource(Origin::root(), r_id, resource));

        // Create proposal (& vote)
        assert_ok!(ChainBridge::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = ChainBridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = chainbridge::ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![],
            status: chainbridge::ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 1,
        };
        assert_eq!(prop, expected);

        // Second relayer votes against
        assert_ok!(ChainBridge::reject_proposal(
            Origin::signed(RELAYER_B),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = ChainBridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = chainbridge::ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![RELAYER_B],
            status: chainbridge::ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 1,
        };
        assert_eq!(prop, expected);

        // Third relayer votes in favour
        assert_ok!(ChainBridge::acknowledge_proposal(
            Origin::signed(RELAYER_C),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = ChainBridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = chainbridge::ProposalVotes {
            votes_for: vec![RELAYER_A, RELAYER_C],
            votes_against: vec![RELAYER_B],
            status: chainbridge::ProposalStatus::Approved,
            expiry: ProposalLifetime::get() + 1,
        };
        assert_eq!(prop, expected);

        assert_eq!(Balances::free_balance(RELAYER_A), ENDOWED_BALANCE + 10);
        assert_eq!(
            Balances::free_balance(ChainBridge::account_id()),
            ENDOWED_BALANCE - 10
        );

        assert_events(vec![
            Event::chainbridge(chainbridge::RawEvent::VoteFor(src_id, prop_id, RELAYER_A)),
            Event::chainbridge(chainbridge::RawEvent::VoteAgainst(
                src_id, prop_id, RELAYER_B,
            )),
            Event::chainbridge(chainbridge::RawEvent::VoteFor(src_id, prop_id, RELAYER_C)),
            Event::chainbridge(chainbridge::RawEvent::ProposalApproved(src_id, prop_id)),
            Event::balances(balances::Event::Transfer(
                ChainBridge::account_id(),
                RELAYER_A,
                10,
            )),
            Event::chainbridge(chainbridge::RawEvent::ProposalSucceeded(src_id, prop_id)),
        ]);
    })
}
