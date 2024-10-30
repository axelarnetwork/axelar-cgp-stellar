use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    InvalidAddress = 1,
    InvalidAmount = 2,
    InsufficientBalance = 3,
    AlreadyInitialized = 4,
    NotInitialized = 5,
}