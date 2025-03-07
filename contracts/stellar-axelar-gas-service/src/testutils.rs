use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};
use stellar_axelar_std::types::Token;

use crate::{AxelarGasService, AxelarGasServiceClient};

pub fn setup_gas_service<'a>(env: &Env) -> AxelarGasServiceClient<'a> {
    let owner: Address = Address::generate(env);
    let operator: Address = Address::generate(env);
    let gas_service_id = env.register(AxelarGasService, (&owner, &operator));
    let gas_service_client = AxelarGasServiceClient::new(env, &gas_service_id);

    gas_service_client
}

pub fn setup_gas_token<'a>(env: &'a Env, sender: &'a Address) -> (Token, TokenClient<'a>) {
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(env));
    let gas_amount: i128 = 1;
    let gas_token = Token {
        address: asset.address(),
        amount: gas_amount,
    };
    let gas_token_client = TokenClient::new(env, &asset.address());

    StellarAssetClient::new(env, &asset.address())
        .mock_all_auths()
        .mint(sender, &gas_amount);

    (gas_token, gas_token_client)
}
