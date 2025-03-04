use soroban_sdk::{contracttype, Address, BytesN, String};
use stellar_axelar_std::contractstorage;

use crate::types::TokenManagerType;

#[contractstorage]
enum DataKey {
    #[instance]
    #[value(Address)]
    Gateway,

    #[instance]
    #[value(Address)]
    GasService,

    #[instance]
    #[value(String)]
    ChainName,

    #[instance]
    #[value(String)]
    ItsHubAddress,

    #[instance]
    #[value(Address)]
    NativeTokenAddress,

    #[instance]
    #[value(BytesN<32>)]
    InterchainTokenWasmHash,

    #[instance]
    #[value(BytesN<32>)]
    TokenManagerWasmHash,

    #[persistent]
    #[status]
    TrustedChain { chain: String },

    #[persistent]
    #[value(TokenIdConfigValue)]
    TokenIdConfig { token_id: BytesN<32> },

    #[persistent]
    #[value(i128)]
    FlowLimit { token_id: BytesN<32> },

    #[temporary]
    #[value(i128)]
    FlowOut { flow_key: FlowKey },

    #[temporary]
    #[value(i128)]
    FlowIn { flow_key: FlowKey },
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
