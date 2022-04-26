use crate::{mock::*, types::*, BuiltinNodes, Config, Error, Nodes};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError, traits::Currency};
use hex_literal::hex;
use sp_core::H256;
use sp_runtime::traits::AtLeast32BitUnsigned;

#[test]
fn it_works() {
	new_test_ext().execute_with(|| {})
}
