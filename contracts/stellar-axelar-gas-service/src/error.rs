use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    InvalidAddress = 2,
    InvalidAmount = 3,
    InsufficientBalance = 4,
}
