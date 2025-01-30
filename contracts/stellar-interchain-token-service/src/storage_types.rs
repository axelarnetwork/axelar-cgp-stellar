use soroban_sdk::{contracttype, Address, BytesN, String};

use crate::types::TokenManagerType;

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Gateway,
    GasService,
    ChainName,
    ItsHubAddress,
    NativeTokenAddress,
    InterchainTokenWasmHash,
    TokenManagerWasmHash,
    TrustedChain(String),
    TokenIdConfig(BytesN<32>),
    FlowLimit(BytesN<32>),
    FlowOut(FlowKey),
    FlowIn(FlowKey),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenIdConfigValue {
    pub token_address: Address,
    pub token_manager: Address,
    pub token_manager_type: TokenManagerType,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct FlowKey {
    pub token_id: BytesN<32>,
    pub epoch: u64,
}
