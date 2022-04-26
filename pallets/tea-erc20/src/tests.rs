use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};

const CENTS: node_primitives::Balance = 10_000_000_000;
const DOLLARS: node_primitives::Balance = 100 * CENTS;

#[test]
fn it_works() {
	new_test_ext().execute_with(|| {})
}
