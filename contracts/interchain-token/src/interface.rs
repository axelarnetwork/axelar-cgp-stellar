use soroban_sdk::token::{self, StellarAssetInterface};
use soroban_sdk::{contractclient, Address, BytesN, Env};

use crate::error::ContractError;

#[allow(dead_code)]
#[contractclient(name = "InterchainTokenClient")]
pub trait InterchainTokenInterface: token::Interface + StellarAssetInterface {
    fn token_id(env: &Env) -> BytesN<32>;

    fn is_minter(env: &Env, minter: Address) -> bool;
    fn mint_from(
        env: &Env,
        minter: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError>;
    fn add_minter(env: &Env, minter: Address);
    fn remove_minter(env: &Env, minter: Address);
}
