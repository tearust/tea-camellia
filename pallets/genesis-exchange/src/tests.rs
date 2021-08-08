use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn tea_to_usd_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let tea_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&user, tea_amount);

		assert_eq!(<Test as Config>::Currency::free_balance(&user), tea_amount);
		assert_eq!(USDStore::<Test>::get(user), 0);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT
		);

		let withdraw_amount = 100;
		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			withdraw_amount
		));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			tea_amount - withdraw_amount
		);
		assert_eq!(USDStore::<Test>::get(user), withdraw_amount);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT + withdraw_amount
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT - withdraw_amount
		);
	})
}

#[test]
fn usd_to_tea_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let usd_amount = 1000;
		USDStore::<Test>::insert(user, usd_amount);

		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);
		assert_eq!(USDStore::<Test>::get(user), usd_amount);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT
		);

		let withdraw_amount = 100;
		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			withdraw_amount
		));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			withdraw_amount
		);
		assert_eq!(USDStore::<Test>::get(user), usd_amount - withdraw_amount);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT - withdraw_amount
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT + withdraw_amount
		);
	})
}
