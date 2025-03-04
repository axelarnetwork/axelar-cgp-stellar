use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    NotMinter = 2,
    InvalidDecimal = 3,
    InvalidTokenName = 4,
    InvalidTokenSymbol = 5,
    InvalidAmount = 6,
    InvalidExpirationLedger = 7,
    InsufficientAllowance = 8,
    InsufficientBalance = 9,
}
