extern crate alloc;

use alloc::vec::Vec;

use soroban_sdk::{token, Address, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_std::ensure;

use crate::error::ContractError;

const NATIVE_TOKEN_NAME: &str = "Stellar";
const NATIVE_TOKEN_SYMBOL: &str = "XLM";
const MAX_DECIMALS: u32 = u8::MAX as u32;
const MAX_NAME_LENGTH: u32 = 32;
const MAX_SYMBOL_LENGTH: u32 = 32;

pub trait TokenMetadataExt: Sized {
    fn new(name: String, symbol: String, decimals: u32) -> Result<Self, ContractError>;

    fn validate(&self) -> Result<(), ContractError>;
}

impl TokenMetadataExt for TokenMetadata {
    fn new(name: String, symbol: String, decimals: u32) -> Result<Self, ContractError> {
        let token_metadata = Self {
            name,
            symbol,
            decimal: decimals,
        };

        token_metadata.validate()?;

        Ok(token_metadata)
    }

    fn validate(&self) -> Result<(), ContractError> {
        ensure!(
            self.decimal <= MAX_DECIMALS,
            ContractError::InvalidTokenDecimals
        );
        ensure!(
            !self.name.is_empty() && self.name.len() <= MAX_NAME_LENGTH,
            ContractError::InvalidTokenName
        );
        ensure!(
            !self.symbol.is_empty() && self.symbol.len() <= MAX_SYMBOL_LENGTH,
            ContractError::InvalidTokenSymbol
        );
        ensure!(is_ascii_string(&self.name), ContractError::InvalidTokenName);
        ensure!(
            is_ascii_string(&self.symbol),
            ContractError::InvalidTokenSymbol
        );

        Ok(())
    }
}

fn is_ascii_string(input: &String) -> bool {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.resize(input.len() as usize, 0);
    input.copy_into_slice(&mut bytes);

    for &byte in bytes.iter() {
        let character = byte as char;
        let is_ascii_char = character.is_ascii();
        if !is_ascii_char {
            return false;
        }
    }
    true
}

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

    TokenMetadata::new(name, symbol, decimals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_metadata_valid_ascii() {
        let env = Env::default();

        let name = String::from_str(&env, "Test");
        let symbol = String::from_str(&env, "Test");
        let decimals = 18;

        let result = TokenMetadata::new(name, symbol, decimals);
        let _ = result.unwrap();
    }

    #[test]
    fn token_metadata_invalid_ascii_name() {
        let env = Env::default();

        let name = String::from_str(&env, "Test世界！");
        let symbol = String::from_str(&env, "Test");
        let decimals = 18;

        let result = TokenMetadata::new(name, symbol, decimals);
        assert!(matches!(result, Err(ContractError::InvalidTokenName)));
    }

    #[test]
    fn token_metadata_invalid_ascii_symbol() {
        let env = Env::default();

        let name = String::from_str(&env, "Test");
        let symbol = String::from_str(&env, "Test世界！");
        let decimals = 18;

        let result = TokenMetadata::new(name, symbol, decimals);
        assert!(matches!(result, Err(ContractError::InvalidTokenSymbol)));
    }
}
