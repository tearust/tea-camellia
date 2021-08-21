use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_new_tapp_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let tapp_name = "test name";
		let init_fund = 1000;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user),
			tapp_name.as_bytes().to_vec(),
			init_fund,
			CurveType::UnsignedLinear,
			CurveType::UnsignedSquareRoot,
		));

		// this is the first tapp so tapp id is 1
		let tapp_id = 1;
		assert_eq!(LastTAppId::<Test>::get(), tapp_id);
		assert_eq!(AccountTable::<Test>::get(user, tapp_id), init_fund);
		assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), init_fund);
		assert_eq!(TAppNames::<Test>::get(tapp_name.as_bytes()), tapp_id);
		let tapp_item = TAppBoundingCurve::<Test>::get(tapp_id);
		assert_eq!(tapp_item.id, tapp_id);
		assert_eq!(tapp_item.buy_curve, CurveType::UnsignedLinear);
		assert_eq!(tapp_item.sell_curve, CurveType::UnsignedSquareRoot);
		assert_eq!(tapp_item.owner, user);
		assert_eq!(&String::from_utf8(tapp_item.name).unwrap(), tapp_name);
		// assert_eq!(<Test as Config>::Currency::free_balance(&user), init_fund);
	})
}
