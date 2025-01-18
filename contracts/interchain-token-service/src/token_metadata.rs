use soroban_sdk::{token, Address, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_std::ensure;
use stellar_axelar_std::token::validate_token_metadata;

use crate::error::ContractError;

const NATIVE_TOKEN_NAME: &str = "Stellar";
const NATIVE_TOKEN_SYMBOL: &str = "XLM";
const MAX_NAME_LENGTH: u32 = 32;

pub fn token_metadata(
    env: &Env,
    token_address: &Address,
    native_token_address: &Address,
) -> Result<TokenMetadata, ContractError> {
    let token = token::Client::new(env, token_address);
    let decimals = token.decimals();
    let name = token.name();
    let symbol = token.symbol();

    // Stellar's native token sets the name and symbol to 'native'. Override it to make it more readable
    let (name, symbol) = if token_address == native_token_address {
        (
            String::from_str(env, NATIVE_TOKEN_NAME),
            String::from_str(env, NATIVE_TOKEN_SYMBOL),
        )
    // If the name is longer than 32 characters, use the symbol as the name to avoid a deployment error on the destination chain
    } else if name.len() > MAX_NAME_LENGTH {
        (symbol.clone(), symbol)
    } else {
        (name, symbol)
    };

    let token_metadata = TokenMetadata {
        name,
        symbol,
        decimal: decimals,
    };

    ensure!(
        validate_token_metadata(&token_metadata).is_ok(),
        ContractError::InvalidTokenMetaData
    );

    Ok(token_metadata)
}
