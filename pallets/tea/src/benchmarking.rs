//! Benchmarking setup for pallet-tea

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use hex_literal::hex;

benchmarks! {
    add_new_node {
        let public = hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), public)
}

impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test,);
