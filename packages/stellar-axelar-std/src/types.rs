use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub address: Address,
    pub amount: i128,
}
