use codec::{Encode};
use frame_support::debug::{self, error};
use frame_support::traits::Randomness;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::{DispatchError, AccountId32};
use pallet_contracts::Config;
use sp_std::vec::Vec;
use core::convert::TryFrom;

// 转换为 AccountId
pub fn to_account_id(account: &[u8]) -> AccountId32 {
	AccountId32::try_from(account).unwrap()
}

pub struct FetchRandomExtension;


impl<C: Config> ChainExtension<C> for FetchRandomExtension {
    fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
        where
            <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        match func_id {
            1101 => {
                debug::info!("run 1001");
                let mut env = env.buf_in_buf_out();
                let random_seed: [u8; 32] = super::RandomnessCollectiveFlip::random_seed().0;
                debug::info!("random_seed: {:?}", random_seed);
                let random_slice = random_seed.encode();
                debug::info!("random_slice: {:?}", random_slice);
                
                env.write(&random_slice, false, None)
                    .map_err(|_| DispatchError::Other("ChainExtension failed to call random"))?;
            }
            1102 => { // 调用 POE 的创建存证接口，无返回
                debug::info!("run 1002");
				let mut env = env.buf_in_buf_out();
				let caller = env.ext().caller().clone();
				debug::info!("caller: {:?}", caller);
				let address = env.ext().address(); // contract
				debug::info!("contract address: {:?}", address);
				let in_len = env.in_len(); // Number of passed parameters
				debug::info!("in_len: {}", in_len);
				let input = env.read(1)?; // Read n incoming parameters
				debug::info!("input with len: {}", input[0]);
                
				let input: Vec<u8> = env.read_as()?;
				let who = to_account_id(caller.as_ref());
				super::PoeModule::do_create_claim(who, input)?;
            }
            1103 => { // 调用 Kitties 模块，创建一个 Kitty
                debug::info!("run 1103");
                let mut env = env.buf_in_buf_out();
                let caller = env.ext().caller().clone();
				let who = to_account_id( caller.as_ref() );

                let kitty_id = super::KittiesModule::create_kitty(&who)?;
                let kitty_id_encoded = kitty_id.encode();
                env.write(&kitty_id_encoded, false, None)
                    .map_err(|_| DispatchError::Other("ChainExtension failed to call create_kitty"))?;
            }
            _ => {
                error!("call an unregistered `func_id`, func_id:{:}", func_id);
                return Err(DispatchError::Other("Unimplemented func_id"));
            }
        }
        Ok(RetVal::Converging(0))
    }

    fn enabled() -> bool {
        true
    }
}