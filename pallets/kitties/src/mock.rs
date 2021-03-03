use crate as pallet_kitties;
use balances;
use sp_core::H256;
use frame_support::{
	parameter_types
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: balances::{Module, Call, Storage, Config<T>, Event<T>},
		KittiesModule: pallet_kitties::{Module, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

type Randomness = pallet_randomness_collective_flip::Module<Test>;

parameter_types! {
	pub const NewKittyReserve: u64 = 5_000;
}
impl pallet_kitties::Config for Test {
	type Event = Event;
	type Randomness = Randomness;
	type KittyIndex = u32;
	type NewKittyReserve = NewKittyReserve;
	type Currency = balances::Module<Self>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	// 因为测试涉及到质押资产，所以需要给一些账户初始化一些钱
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	balances::GenesisConfig::<Test> {
		// Provide some initial balances
		balances: vec![(1, 10000000000), (2, 110000000), (3, 1200000000), (4, 1300000000), (5, 1400000000)],
	}.assimilate_storage(&mut t)
	.unwrap();
	let mut ext: sp_io::TestExternalities = t.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}