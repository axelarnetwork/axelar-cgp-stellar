use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub address: Address, // TODO: check if this can be changed to a TokenClient type instead which is richer than Address, or a generic type implementing TokenInterface
    pub amount: i128,
}
