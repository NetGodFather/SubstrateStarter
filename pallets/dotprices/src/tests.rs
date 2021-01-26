use crate::{mock::*};
use frame_support::{assert_ok};

#[test]
fn update_price_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let price:u64 = (18.21 * 10000f64) as u64;
		assert_ok!(DotPrices::update_price(Origin::signed(1), price ) );
		// assert_eq!(DotPrices::<T>::get(0), Some(price));
	});
}