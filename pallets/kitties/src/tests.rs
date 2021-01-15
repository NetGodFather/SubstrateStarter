use crate::{Event, Error, mock::*};
use frame_support::{assert_noop, assert_ok, traits::{OnFinalize, OnInitialize}};

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

		// 因为有质押，所以会触发两个事件，这里只监控第二个
		assert_eq!(
			System::events()[1].event,
			TestEvent::kitties( Event::<Test>::Created( 1u64 , 0) )
		);
	})
}

// 测试创建 Kitty 失败，因为没有质押的钱
#[test]
fn create_kitty_failed_when_not_enough_money(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_noop!(KittiesModule::create( Origin::signed(9)) , Error::<Test>::MoneyNotEnough);
	})
}

// 测试转让 Kitty 成功
#[test]
fn transfer_kitty_works(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_ok!(KittiesModule::transfer( Origin::signed(1), 2, 0 ) );

		// 因为有创建时候的（质押+创建），转让时候的（质押+解质押+转让），所以总共会触发个五个事件，这里只监控第五个
		assert_eq!(
			System::events()[4].event,
			TestEvent::kitties( Event::<Test>::Transferred( 1u64 ,2u64, 0) )
		);
	});
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
		
		// 因为有质押，所以会触发两个事件，这里只监控第二个
		assert_eq!(
			System::events()[1].event,
			TestEvent::kitties( Event::<Test>::Created( 1u64 , 0) )
		);
	});
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
// 挂单价格成功
#[test]
fn ask_price_work(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );
		assert_ok!( KittiesModule::ask( Origin::signed(1), 0, Some(5_000_000_000) ) );
	});
}

// 挂单价格失败，因为不是同一个用户
#[test]
fn ask_price_fail_when_not_owner(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );
		assert_noop!( KittiesModule::ask( Origin::signed(2), 0, Some(5_000_000_000) ) , Error::<Test>::NotKittyOwner);
	});
}