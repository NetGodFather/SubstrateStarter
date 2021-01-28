#![cfg_attr(not(feature = "std"), no_std)]

use core::{convert::TryInto};
use frame_support::{debug, decl_module, decl_storage, decl_event, decl_error, StorageValue};
use sp_std::{ prelude::*, str, collections::vec_deque::VecDeque};
use sp_core::crypto::KeyTypeId;
use frame_system::{
	ensure_signed,
	offchain::{AppCrypto, CreateSignedTransaction, Signer, SendSignedTransaction},
};
use sp_runtime::{
	offchain as rt_offchain,
};
use serde_json::{Value};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

pub const PRICES_VEC_LEN: usize = 10;
// 获取报价的地址
pub const HTTP_REMOTE_REQUEST: &str = "https://api.coincap.io/v2/assets/polkadot";
// HTTP请求超时时间，单位毫秒
pub const FETCH_TIMEOUT_PERIOD: u64 = 3000;

// 用于 OCW 签名的密钥类型标识，在 node/src/service.rs 中会需要用到
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");

// 指定的签名的 mod 
pub mod crypto {
	// cate 是指当前编译的单元（基本就是这个 rs 文件），cate::KEY_TYPE 就是指编译单元定义的变量，就是引入第 52 行定义的。
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	use sp_runtime::{
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	
	app_crypto!(sr25519, KEY_TYPE);

	// 在 runtime/src/lib.rs 中指定 pallet 的 AuthorityId 时候需要
	pub struct AuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// 实现正式的 runtime 的签名【 AppCrypto 的泛型约束用 sp_runtime 里边的】
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for AuthId
	{
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
		pub DotPrices get(fn dot_prices): VecDeque<Option<Vec<u8>>>;
    }
}

decl_event!(
	// where 后边的部分，是表示在 Event 里边需要用的一些类型来自哪个 Trait 定义
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId, {
		PriceUpdated(AccountId, Vec<u8>),
    }
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		OffchainSignedTxError,
		NoLocalAcctForSigning,
		UnknownOffchainMux,
		HttpFetchingError,	// 通过HTTP获取数据出错
		FetchPriceError,
    }
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// 如果有触发错误信息，必须包含这一行
		type Error = Error<T>;
		// 如果有触发事件，必须包含这一行
		fn deposit_event() = default;

		#[weight = 0]
		pub fn update_price(origin, price: Vec<u8>){
			let sender = ensure_signed(origin)?;
			
			DotPrices::mutate(|prices| {
				if prices.len() == PRICES_VEC_LEN {
					let _ = prices.pop_front();
				}
				prices.push_back(Some(price.clone()));
			});

			Self::deposit_event(RawEvent::PriceUpdated(sender, price));
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("Entering off-chain worker");
			const TX_TYPES: u32 = 4;
			let modu = block_number.try_into().map_or(TX_TYPES, |bn: usize| (bn as u32) % TX_TYPES);
			let result = match modu {
				0 => Self::offchain_signed_update_price(),
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
	fn offchain_signed_update_price() -> Result<(), Error<T>> {
		// 提取签名账号
		let signer = Signer::<T, T::AuthorityId>::any_account();

		let price:Vec<u8> = Self::fetch_dot_price().map_err(|_| <Error<T>>::FetchPriceError)?;

		let result = signer.send_signed_transaction(|_acct|
			// This is the on-chain function
			Call::update_price(price.clone())
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
	// 获取价格信息，然后调用接口更新
	fn fetch_dot_price() -> Result<Vec<u8>, Error<T>> {
		// 读取从网页获取的数据，如果失败报错
		let resp_bytes = Self::fetch_from_remote().map_err(|e| {
			debug::error!("fetch_from_remote error: {:?}", e);
			<Error<T>>::HttpFetchingError
		})?;
		
		// 转换为字符串，如果转换不成功，报错
		let resp_str = str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
		debug::info!("resp_str {}", resp_str);
		
		let asset: Value = serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingError)?;
		let price : Vec<u8> = asset["data"]["priceUsd"].as_str().unwrap().as_bytes().to_vec();

		debug::info!("price is {:?}", price);
		return Ok(price);
	}
	// 从远程网页上获取数据
	fn fetch_from_remote() -> Result<Vec<u8>, Error<T>>{
		debug::info!("sending request to: {}", HTTP_REMOTE_REQUEST);
		
		let request = rt_offchain::http::Request::get(HTTP_REMOTE_REQUEST);

		// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
		// 保持 OCW 执行时间合理，将调用限制在3秒内（使用 sp_io::offchain::timestamp() 加上一个 ）
		//  sp_io::offchain::timestamp() 返回当前 UNIX 的毫秒，类型是 Timestamp  .add 方法增加一个时间
		// rt_offchain::Duration::from_millis(FETCH_TIMEOUT_PERIOD) 通过指定的毫秒数，返回一个持续时间数据 Duration
		// 
		let timeout = sp_io::offchain::timestamp()
			.add(rt_offchain::Duration::from_millis(FETCH_TIMEOUT_PERIOD));

		// For github API request, we also need to specify `user-agent` in http request header.
		//   See: https://developer.github.com/v3/#user-agent-required
		// 通过 github API 请求数据，我们需要增加 user-agen 
		// .deadline 设置超时时间（这里是时间戳，不是时间间隔）
		// .send 真正发出请求，此操作如果成功，将返回一个 PendingRequest ，异步获得数据
		// .map_err 处理错误
		let pending = request
			.deadline(timeout) // Setting the timeout time
			.send() // Sending the request out by the host
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		// By default, the http request is async from the runtime perspective. So we are asking the
		//   runtime to wait here.
		// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
		//   ref: https://substrate.dev/rustdocs/v2.0.0/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
		// 默认情况下，从运行时的角度来看，http请求是异步的。所以我们要求运行时在这里等待。
		// 这里的返回值是' Result '的' Result '，所以我们用两个' ? '将它展开两次。
		// 因为 try_wait 会有两层的错误，第一层是超时被终止，和请求的内容无关。另外一层是属于网络请求的时候被中断发生了
		// 错误，所以如果想要直接获得最终的结果，需要两层的 map_err 来获取成功请求的 response 
		// 其实还有第三层就是服务器响应的错误，是通过 response.code 来识别的，在后边会判断
		let response = pending
			.try_wait(timeout)
			.map_err(|_| <Error<T>>::HttpFetchingError)?
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		// 如果返回的状态吗不是 200 
		if response.code != 200 {
			debug::error!("Unexpected http request status code: {}", response.code);
			return Err(<Error<T>>::HttpFetchingError);
		}
		// 如果一切都OK，那么将返回的内容放到 Vec 里边去
		Ok(response.body().collect::<Vec<u8>>())
	}
}