use soroban_sdk::token::TokenClient;
use soroban_sdk::{Address, Env};
use stellar_token_manager::TokenManagerClient;

use crate::error::ContractError;
use crate::storage::TokenIdConfigValue;
use crate::token_manager::TokenManagerClientExt;
use crate::types::TokenManagerType;

pub fn take_token(
    env: &Env,
    sender: &Address,
    TokenIdConfigValue {
        token_address,
        token_manager,
        token_manager_type,
    }: TokenIdConfigValue,
    amount: i128,
) -> Result<(), ContractError> {
    let token = TokenClient::new(env, &token_address);

    match token_manager_type {
        TokenManagerType::NativeInterchainToken => token.burn(sender, &amount),
        TokenManagerType::LockUnlock => token.transfer(sender, &token_manager, &amount),
    }

    Ok(())
}

pub fn give_token(
    env: &Env,
    recipient: &Address,
    TokenIdConfigValue {
        token_address,
        token_manager,
        token_manager_type,
    }: TokenIdConfigValue,
    amount: i128,
) -> Result<(), ContractError> {
    let token_manager = TokenManagerClient::new(env, &token_manager);

    match token_manager_type {
        TokenManagerType::NativeInterchainToken => {
            token_manager.mint(env, &token_address, recipient, amount)
        }
        TokenManagerType::LockUnlock => {
            token_manager.transfer(env, &token_address, recipient, amount)
        }
    }

    Ok(())
}
