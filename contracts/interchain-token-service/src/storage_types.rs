use soroban_sdk::{contracttype, Address, BytesN, String};

use crate::types::TokenManagerType;

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    TrustedChain(String),
    Gateway,
    GasService,
    ItsHubAddress,
    ChainName,
    InterchainTokenWasmHash,
    TokenIdConfigKey(BytesN<32>),
    FlowLimit(BytesN<32>),
    FlowOut(BytesN<32>, u64),
    FlowIn(BytesN<32>, u64),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenIdConfigValue {
    pub token_address: Address,
    pub token_manager_type: TokenManagerType,
}
