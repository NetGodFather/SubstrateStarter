#![cfg_attr(not(feature = "std"), no_std)]

use core::{convert::TryInto};
use frame_support::{debug, decl_module, decl_storage, decl_event, decl_error, StorageValue };
use sp_std::{ collections::vec_deque::VecDeque};
use sp_core::crypto::KeyTypeId;
use frame_system::{
	ensure_signed,
	offchain::{AppCrypto, CreateSignedTransaction, Signer, SendSignedTransaction},
};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

pub const PRICES_VEC_LEN: usize = 10;

// 用于 OCW 签名的密钥类型标识，在 node/src/service.rs 中会需要用到
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");

// 指定的签名的 mod 
pub mod crypto {
	// cate 是指当前编译的单元（基本就是这个 rs 文件），cate::KEY_TYPE 就是指编译单元定义的变量，就是引入第 52 行定义的。
	use crate::KEY_TYPE;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	use sp_runtime::{
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE);

	// 在 runtime/src/lib.rs 中指定 pallet 的 AuthorityId 时候需要
	pub struct AuthId;
	// 实现正式的 runtime 的签名【 AppCrypto 的泛型约束用 sp_runtime 里边的】
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

// 因为 OCW 需要发起签名的交易，所以需要实现 CreateSignedTransaction 
pub trait Trait: frame_system::Trait + CreateSignedTransaction<Call<Self>> {
	/// The identifier type for an offchain worker.
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;
	/// The overarching event type.
    // 如果有触发事件，就必须包含这一行
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	// T: Trait 里边的 Trait 就是第17行定义的 Trait
	trait Store for Module<T: Trait> as DotPrices {
		pub DotPrices get(fn dot_prices): VecDeque<Option<u64>>;
    }
}

decl_event!(
	// where 后边的部分，是表示在 Event 里边需要用的一些类型来自哪个 Trait 定义
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId, {
		PriceUpdated(AccountId, u64),
    }
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		OffchainSignedTxError,
		NoLocalAcctForSigning,
		UnknownOffchainMux,
    }
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// 如果有触发错误信息，必须包含这一行
		type Error = Error<T>;
		// 如果有触发事件，必须包含这一行
		fn deposit_event() = default;

		#[weight = 0]
		pub fn update_price(origin, price: u64){
			let sender = ensure_signed(origin)?;
			DotPrices::mutate(|prices| {
				if prices.len() == PRICES_VEC_LEN {
					let _ = prices.pop_front();
				}
				prices.push_back(Some(price));
			});

			Self::deposit_event(RawEvent::PriceUpdated(sender, price));
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("Entering off-chain worker");
			const TX_TYPES: u32 = 4;
			let price: u64 = 180000u64 + (block_number.try_into().map_or(TX_TYPES, |bn: usize| (bn as u32 ) % 99) as u64);
			let modu = block_number.try_into().map_or(TX_TYPES, |bn: usize| (bn as u32) % TX_TYPES);
			let result = match modu {
				0 => Self::offchain_signed_update_price(price),
				1 => 
				_ => Err(Error::<T>::UnknownOffchainMux),
			};

			if let Err(e) = result {
				debug::error!("offchain_worker error: {:?}", e);
			}
		}
    }
}

impl<T: Trait> Module<T> {
	//  OC 签名发起交易
	fn offchain_signed_update_price(price: u64) -> Result<(), Error<T>> {
		// 提取签名账号
		let signer = Signer::<T, T::AuthorityId>::any_account();

		let result = signer.send_signed_transaction(|_acct|
			// This is the on-chain function
			Call::update_price(price)
		);

		// Display error if the signed tx fails.
		if let Some((acc, res)) = result {
			if res.is_err() {
				debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
				return Err(<Error<T>>::OffchainSignedTxError);
			}
			// Transaction is sent successfully
			return Ok(());
		}

		Err(<Error<T>>::NoLocalAcctForSigning)
	}
}