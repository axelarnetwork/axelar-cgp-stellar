use soroban_sdk::{Env, IntoVal};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gas_service::testutils::setup_gas_service;
use stellar_axelar_gas_service::AxelarGasServiceClient;
use stellar_axelar_gateway::testutils::{setup_gateway, TestSignerSet};
use stellar_axelar_gateway::AxelarGatewayClient;

use crate::testutils::setup_its;
use crate::InterchainTokenServiceClient;

pub fn setup_env<'a>() -> (
    Env,
    InterchainTokenServiceClient<'a>,
    AxelarGatewayClient<'a>,
    AxelarGasServiceClient<'a>,
    TestSignerSet,
) {
    let env = Env::default();

    let (signers, gateway_client) = setup_gateway(&env, 0, 5);
    let gas_service_client: AxelarGasServiceClient<'_> = setup_gas_service(&env);

    let client = setup_its(&env, &gateway_client, &gas_service_client, None);

    (env, client, gateway_client, gas_service_client, signers)
}

pub trait TokenMetadataExt {
    fn new(env: &Env, name: &str, symbol: &str, decimal: u32) -> Self;
}

impl TokenMetadataExt for TokenMetadata {
    fn new(env: &Env, name: &str, symbol: &str, decimal: u32) -> Self {
        Self {
            decimal,
            name: name.into_val(env),
            symbol: symbol.into_val(env),
        }
    }
}

pub const INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX: i32 = -4;
pub const INTERCHAIN_TOKEN_DEPLOYED_WITHOUT_GAS_TOKEN_EVENT_IDX: i32 = -2;
pub const INTERCHAIN_TOKEN_DEPLOYED_NO_SUPPLY_EVENT_IDX: i32 =
    INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX + 1;
pub const TOKEN_MANAGER_DEPLOYED_EVENT_IDX: i32 = INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX + 1;
