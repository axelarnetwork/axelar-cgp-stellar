use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub address: Address,
    pub amount: i128,
}

pub struct TokenWithEnv {
    pub env: Env,
    pub address: Address,
    pub amount: i128,
}
