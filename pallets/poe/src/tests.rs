
use crate::{Error, mock::*};
use super::*;
use frame_support::{assert_noop, assert_ok};

// 测试存证成功添加的情况
#[test]
fn create_claim_works(){
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		assert_eq!(Proofs::<Test>::get(&claim),(1, system::Module::<Test>::block_number()));
	})
}

// 测试存证已经存在添加失败的情况
#[test]
fn create_claim_failed_when_claim_already_exist(){
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExists
		);
	})
}

// 测试存证因为太长添加失败的情况，Mock 里边将长度设置为了 6
#[test]
fn create_claim_failed_when_claim_is_too_long(){
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3, 4, 5, 6];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofTooLong
		);
	})
}

// 测试正常移除存证的情况
#[test]
fn revoke_claim_works(){
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_ok!( PoeModule::revoke_claim(Origin::signed(1), claim.clone()) );
	})
}

// 测试移除的存证不存在的情况
#[test]
fn revoke_claim_failed_when_claim_is_not_exist(){
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofNotExists
		);
	})
}
// 测试移除的存证不属于调用者的情况
#[test]
fn revoke_claim_failed_with_wrong_owner(){
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}

// 测试存证正常转移的情况
#[test]
fn transfer_claim_works(){
	new_test_ext().execute_with(||{
		let claim: Vec<u8> = vec![0,1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		let _ = PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2);

		let (owner, _) = Proofs::<Test>::get(&claim);

		assert_eq!( owner, 2);
	})
}

// 测试存证转移因为不存在失败的情况
#[test]
fn transfer_claim_failed_when_not_exists(){
	new_test_ext().execute_with(||{
		let claim: Vec<u8> = vec![0,1];
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2),
			Error::<Test>::ProofNotExists
		);
	})
}
// 测试存证转移因为调用者不是拥有者，失败的情况
#[test]
fn transfer_claim_failed_when_not_owner(){
	new_test_ext().execute_with(||{
		let claim: Vec<u8> = vec![0,1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), claim.clone(), 3),
			Error::<Test>::NotClaimOwner
		);
	})
}