use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gas_service::AxelarGasServiceClient;
use stellar_axelar_gateway::AxelarGatewayClient;

use crate::{InterchainTokenService, InterchainTokenServiceClient};

// Note: On changes to `interchain-token` and `token-manager` crates, recompile via `stellar contract build && ./optimize.sh`
// and copy the built `target/wasm32-unknown-unknown/release/stellar_*.optimized.wasm` to ../testdata.
pub const INTERCHAIN_TOKEN_WASM: &[u8] =
    include_bytes!("./testdata/stellar_interchain_token.optimized.wasm");
pub const TOKEN_MANAGER_WASM: &[u8] =
    include_bytes!("./testdata/stellar_token_manager.optimized.wasm");

pub fn setup_its<'a>(
    env: &Env,
    gateway: &AxelarGatewayClient,
    gas_service: &AxelarGasServiceClient,
    chain_name: Option<String>,
) -> InterchainTokenServiceClient<'a> {
    let owner = Address::generate(env);
    let operator = Address::generate(env);
    let its_hub_address = String::from_str(env, "its_hub_address");
    let chain_name = chain_name.unwrap_or_else(|| String::from_str(env, "chain_name"));

    let interchain_token_wasm_hash = env.deployer().upload_contract_wasm(INTERCHAIN_TOKEN_WASM);

    let token_manager_wasm_hash = env.deployer().upload_contract_wasm(TOKEN_MANAGER_WASM);

    let native_token_address = env.register_stellar_asset_contract_v2(owner.clone());

    let contract_id = env.register(
        InterchainTokenService,
        (
            &owner,
            &operator,
            &gateway.address,
            &gas_service.address,
            its_hub_address,
            chain_name,
            native_token_address.address(),
            interchain_token_wasm_hash,
            token_manager_wasm_hash,
        ),
    );

    InterchainTokenServiceClient::new(env, &contract_id)
}

pub fn setup_its_token(
    env: &Env,
    client: &InterchainTokenServiceClient,
    sender: &Address,
    supply: i128,
) -> (BytesN<32>, TokenMetadata) {
    let salt: BytesN<32> = BytesN::from_array(env, &[1u8; 32]);
    let token_metadata = TokenMetadata {
        name: String::from_str(env, "Test"),
        symbol: String::from_str(env, "TEST"),
        decimal: 18,
    };

    let token_id = client.mock_all_auths().deploy_interchain_token(
        sender,
        &salt,
        &token_metadata,
        &supply,
        &None,
    );

    (token_id, token_metadata)
}
