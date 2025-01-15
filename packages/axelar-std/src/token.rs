use crate::ensure;
use soroban_sdk::contracterror;
use soroban_token_sdk::metadata::TokenMetadata;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    InvalidDecimal = 0,
    InvalidTokenName = 1,
    InvalidTokenSymbol = 2,
}

pub fn validate_token_metadata(token_metadata: &TokenMetadata) -> Result<(), TokenError> {
    ensure!(
        token_metadata.decimal <= u8::MAX.into(),
        TokenError::InvalidDecimal
    );
    ensure!(
        !token_metadata.name.is_empty(),
        TokenError::InvalidTokenName
    );
    ensure!(
        !token_metadata.symbol.is_empty(),
        TokenError::InvalidTokenSymbol
    );
    Ok(())
}
