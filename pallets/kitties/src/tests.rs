use crate::{Event, Error, mock::*};
use frame_support::{assert_noop, assert_ok, traits::{OnFinalize, OnInitialize}};
use frame_system::{EventRecord, Phase};

fn run_to_block( n: u64) {
	while System::block_number() < n {
		KittiesModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number()+1);
		System::on_initialize(System::block_number());
		KittiesModule::on_initialize(System::block_number());
	}
}

// 测试创建一个 Kitty
#[test]
fn create_kitty_works(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create( Origin::signed(1)) );

		assert_eq!(
			System::events(),
			vec![EventRecord {
				phase: Phase::Initialization,
				event: TestEvent::kitties_event( Event::Created( 1 as u64 , 0) ),
				topics: vec![],
			}]
		);
	})
}

// 测试转让 Kitty 成功
#[test]
fn transfer_kitty_works(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_ok!(KittiesModule::transfer( Origin::signed(1), 2, 0 ) );
	})
}

// 测试转让 Kitty 失败，因为 kitty 不存在
#[test]
fn transfer_kitty_failed_when_not_exists(){
	new_test_ext().execute_with(|| {
		assert_noop!(KittiesModule::transfer( Origin::signed(1), 2, 0 ) , Error::<Test>::KittyNotExists);
	})
}

// 测试转让 Kitty 失败，因为不是 kitty 拥有者
#[test]
fn transfer_kitty_failed_when_not_owner(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!(KittiesModule::transfer( Origin::signed(2), 3, 0 ) , Error::<Test>::NotKittyOwner);
	})
}

// 测试转让 Kitty 失败，因为不能转给自己
#[test]
fn transfer_kitty_failed_when_transfer_self(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!(KittiesModule::transfer( Origin::signed(1), 1, 0 ) , Error::<Test>::TransferToSelf);
	})
}
// 测试繁殖成功
#[test]
fn breed_kitty_work(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_ok!( KittiesModule::breed( Origin::signed(1), 0, 1 ) );
	})
}

// 测试繁殖失败，因为猫相同
#[test]
fn breed_kitty_fail_when_same(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!( KittiesModule::breed( Origin::signed(1), 0, 0 ) , Error::<Test>::RequiredDiffrentParent);
	})
}

// 测试繁殖失败，因为猫不存在
#[test]
fn breed_kitty_fail_when_not_exists(){
	new_test_ext().execute_with(|| {
		assert_noop!( KittiesModule::breed( Origin::signed(1), 0, 1 ) , Error::<Test>::KittyNotExists);
	})
}

// 测试繁殖失败，因为不是猫的主人
#[test]
fn breed_kitty_fail_when_(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!( KittiesModule::breed( Origin::signed(2), 0, 1) , Error::<Test>::NotKittyOwner);
	})
}