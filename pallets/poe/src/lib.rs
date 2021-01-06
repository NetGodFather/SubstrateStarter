#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;
use frame_support::traits::Get;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	// 因为需要触发时间，所以需要有这一行
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	// 设置存证最大长度，这样设置，可以在 runtime 里边去配置具体长度
	type MaxClaimLength: Get<u32>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as PoeModule {
		// 存证保存的变量，十一二点 hashmap ，KEY 是存证内容的，会用 blake2_128_concat 就行 HASH ，只是一个元组
		Proofs get(fn proofs): map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		// 触发的事件，包括：创建、删除、转让
		ClaimCreated(AccountId, Vec<u8>),
		ClaimRemoved(AccountId, Vec<u8>),
		ClaimTransfered(AccountId, Vec<u8>, AccountId),
	}
);

// Errors inform users that something went wrong.

decl_error! {
	pub enum Error for Module<T: Trait> {
		// 包含的错误信息，包括：存证已经存在、存证不存在，不是存证的主人，存证太长
		ProofAlreadyExists,
		ProofNotExists,
		NotClaimOwner,
		ProofTooLong,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		// 创建存证
		#[weight = 10_000]
		pub fn create_claim(origin, claim: Vec<u8>) -> dispatch::DispatchResult {
			// 验签+获得调用者
			let sender = ensure_signed(origin)?;
			// 检测存证是否已经存在
			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExists);
			// 检测存证的长度是否超过限制
			ensure!( T::MaxClaimLength::get() >= claim.len() as u32, Error::<T>::ProofTooLong);
			// 存入存证数据
			Proofs::<T>::insert(&claim,(sender.clone(), system::Module::<T>::block_number()));
			// 触发存证写入成功的时间
			Self::deposit_event(RawEvent::ClaimCreated(sender, claim));
			Ok(())
		}
		// 存证删除
		#[weight = 10_000]
		pub fn revoke_claim(origin, claim: Vec<u8>) -> dispatch::DispatchResult {
			// 验签+获得调用者
			let sender = ensure_signed(origin)?;
			// 检测存证是否存在，如果不存在报错
			ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::ProofNotExists);
			// 读取存证信息，主要是需要获得存证的拥有者，block_number 前边加 _ 表示这个变量后边不用，否则编译器会报警告
			let (owner, _block_number) = Proofs::<T>::get(&claim);
			// 如果存证的拥有者和调用者不是一个人，报错
			ensure!(owner == sender, Error::<T>::NotClaimOwner);
			// 删除存证
			Proofs::<T>::remove(&claim);
			// 触发存证删除的事件
			Self::deposit_event(RawEvent::ClaimRemoved(sender, claim));
			Ok(())
		}
		// 存证转让
		#[weight = 10_000]
		pub fn transfer_claim(origin, claim: Vec<u8>, receiver: T::AccountId) -> dispatch::DispatchResult {
			// 验签+获得调用者
			let sender = ensure_signed(origin)?;
			// 检测存证是否存在，如果不存在，报错
			ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::ProofNotExists);
			// 获取存证信息，主要是需要获得存证拥有者
			let (owner, _block_number) = Proofs::<T>::get(&claim);
			// 检测拥有者和调用者是不是一个人，如果不是报错
			ensure!( owner == sender , Error::<T>::NotClaimOwner);
			// 更新存证信息，将拥有人修改为指定的账号，用 insert ，因为是 hashmap 会自动实现为替换
			Proofs::<T>::insert(&claim,(receiver.clone(), system::Module::<T>::block_number()));
			// 触发存证转让的事件
			Self::deposit_event(RawEvent::ClaimTransfered(sender, claim, receiver));
			Ok(())
		}
	}
}
