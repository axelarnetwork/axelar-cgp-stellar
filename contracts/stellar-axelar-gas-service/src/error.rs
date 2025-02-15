use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    InvalidAddress = 2,
    InvalidAmount = 3,
    InsufficientBalance = 4,
}
