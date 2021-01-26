#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, StorageValue };
use frame_system::ensure_signed;
use sp_std::{ collections::vec_deque::VecDeque};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const PRICES_VEC_LEN: usize = 10;

pub trait Trait: frame_system::Trait {
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
    }
}