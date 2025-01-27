use soroban_sdk::BytesN;
use soroban_sdk::{contracttype, Address};
use stellar_axelar_std::contractstorage;
use stellar_interchain_token_service::types::TokenManagerType;

#[contractstorage]
#[derive(Clone, Debug)]
pub enum DataKey {
    #[instance]
    #[value(Address)]
    Gateway,

    #[instance]
    #[value(Address)]
    GasService,

    #[instance]
    #[value(Address)]
    InterchainTokenService,

    #[temporary]
    #[value(Address)]
    Users { user: Address },

    #[persistent]
    #[value(TokenIdConfig)]
    TokenIdConfigs { token_id: BytesN<32> },
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenIdConfig {
    pub token_address: Address,
    pub token_manager_type: TokenManagerType,
}
