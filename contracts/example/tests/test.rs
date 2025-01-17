#![cfg(test)]
extern crate std;

use example::{Example, ExampleClient};
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _};
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, IntoVal, String, Symbol};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gas_service::{AxelarGasService, AxelarGasServiceClient};
use stellar_axelar_gateway::event::ContractCalledEvent;
use stellar_axelar_gateway::testutils::{
    self, deterministic_rng, generate_proof, generate_test_message_with_rng, get_approve_hash,
    TestSignerSet,
};
use stellar_axelar_gateway::types::Message;
use stellar_axelar_gateway::AxelarGatewayClient;
use stellar_axelar_std::traits::BytesExt;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{auth_invocation, events};
use stellar_interchain_token_service::{InterchainTokenService, InterchainTokenServiceClient};

const INTERCHAIN_TOKEN_WASM_HASH: &[u8] = include_bytes!("./testdata/interchain_token.wasm");

fn setup_gateway<'a>(env: &Env) -> (TestSignerSet, AxelarGatewayClient<'a>) {
    let (signers, client) = testutils::setup_gateway(env, 0, 5);
    (signers, client)
}

fn setup_gas_service<'a>(env: &Env) -> (AxelarGasServiceClient<'a>, Address, Address) {
    let owner: Address = Address::generate(env);
    let gas_collector: Address = Address::generate(env);
    let gas_service_id = env.register(AxelarGasService, (&owner, &gas_collector));
    let gas_service_client = AxelarGasServiceClient::new(env, &gas_service_id);

    (gas_service_client, gas_collector, gas_service_id)
}

fn setup_app<'a>(
    env: &Env,
    gateway: &Address,
    gas_service: &Address,
    interchain_token_service: &Address,
) -> ExampleClient<'a> {
    let id = env.register(Example, (gateway, gas_service, interchain_token_service));
    let client = ExampleClient::new(env, &id);

    client
}

fn setup_interchain_token_service<'a>(
    env: &Env,
    gateway: &Address,
    gas_service: &Address,
) -> (InterchainTokenServiceClient<'a>, Address) {
    let owner = Address::generate(env);
    let operator = Address::generate(env);
    let its_hub_address = String::from_str(env, "its_hub_address");
    let chain_name = String::from_str(env, "chain_name");

    let interchain_token_wasm_hash = env
        .deployer()
        .upload_contract_wasm(INTERCHAIN_TOKEN_WASM_HASH);

    let interchain_token_service_id = env.register(
        InterchainTokenService,
        (
            &owner,
            &operator,
            gateway,
            gas_service,
            its_hub_address,
            chain_name,
            interchain_token_wasm_hash,
        ),
    );

    let interchain_token_service_client =
        InterchainTokenServiceClient::new(env, &interchain_token_service_id);

    (interchain_token_service_client, interchain_token_service_id)
}

fn setup_interchain_token_service_token(
    env: &Env,
    client: &InterchainTokenServiceClient,
    sender: &Address,
    supply: i128,
) -> BytesN<32> {
    let salt = BytesN::from_array(env, &[1u8; 32]);
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

    token_id
}

fn setup_gas_token(env: &Env, sender: &Address) -> Token {
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(env));
    let gas_amount: i128 = 1;
    let gas_token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    StellarAssetClient::new(env, &asset.address())
        .mock_all_auths()
        .mint(sender, &gas_amount);

    gas_token
}

#[test]
fn gmp_example() {
    let env = Env::default();
    env.mock_all_auths();

    let user: Address = Address::generate(&env);

    // Setup source Axelar gateway
    let source_chain = String::from_str(&env, "source");
    let (_, source_gateway_client) = setup_gateway(&env);
    let source_gateway_id = source_gateway_client.address;
    let (source_gas_service_client, _source_gas_collector, source_gas_service_id) =
        setup_gas_service(&env);
    let (interchain_token_service_client, interchain_token_service_id) =
        setup_interchain_token_service(&env, &source_gateway_id, &source_gas_service_id);

    let source_app = setup_app(
        &env,
        &source_gateway_id,
        &source_gas_service_id,
        &interchain_token_service_id,
    );

    // Setup destination Axelar gateway
    let destination_chain = String::from_str(&env, "destination");
    let (signers, destination_gateway_client) = setup_gateway(&env);

    let (_destination_gas_service_client, _destination_gas_collector, destination_gas_service_id) =
        setup_gas_service(&env);
    let destination_app = setup_app(
        &env,
        &destination_gateway_client.address,
        &destination_gas_service_id,
        &interchain_token_service_id,
    );

    // Set cross-chain message params
    let source_address = source_app.address.to_string();
    let destination_address = destination_app.address.to_string();
    let (_, payload) = generate_test_message_with_rng(&env, deterministic_rng());
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    // Initiate cross-chain contract call, sending message from source to destination
    let asset = &env.register_stellar_asset_contract_v2(user.clone());
    let asset_client = StellarAssetClient::new(&env, &asset.address());
    let gas_amount: i128 = 100;
    let gas_token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    asset_client.mint(&user, &gas_amount);

    source_app.send(
        &user,
        &destination_chain,
        &destination_address,
        &payload,
        &gas_token,
    );

    let transfer_auth = auth_invocation!(
        &env,
        user,
        asset_client.transfer(&user, source_gas_service_id, gas_token.amount)
    );

    let pay_gas_auth = auth_invocation!(
        &env,
        user,
        source_gas_service_client.pay_gas(
            source_app.address.clone(),
            destination_chain.clone(),
            destination_address.clone(),
            payload.clone(),
            &user,
            gas_token.clone(),
            &Bytes::new(&env)
        ),
        transfer_auth
    );

    let send_auth = auth_invocation!(
        &env,
        user,
        source_app.send(
            &user,
            destination_chain.clone(),
            destination_address,
            payload.clone(),
            gas_token
        ),
        pay_gas_auth
    );

    assert_eq!(env.auths(), send_auth);

    // Axelar hub confirms the contract call, i.e Axelar verifiers verify/vote on the emitted event
    let message_id = String::from_str(&env, "test");

    // Confirming message from source Axelar gateway
    goldie::assert!(events::fmt_last_emitted_event::<ContractCalledEvent>(&env));

    // Axelar hub signs the message approval, Signing message approval for destination
    let messages = vec![
        &env,
        Message {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
            source_address: source_address.clone(),
            contract_address: destination_app.address.clone(),
            payload_hash,
        },
    ];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    // Submitting signed message approval to destination Axelar gateway
    destination_gateway_client.approve_messages(&messages, &proof);

    // Executing message on destination app
    destination_app.execute(&source_chain, &message_id, &source_address, &payload);

    let gas_token = setup_gas_token(&env, &user);
    let amount = 1000;
    let token_id =
        setup_interchain_token_service_token(&env, &interchain_token_service_client, &user, 1000);
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = Some(Bytes::from_hex(&env, "abcd"));

    // Register destination chain as trusted
    interchain_token_service_client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    // Add authorization for the token transfer
    source_app.mock_all_auths().send_token(
        &user,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &gas_token,
    );

    // Test token receiving functionality
    let received_token_id = BytesN::<32>::random(&env);
    let received_token_address = Address::generate(&env);
    let received_amount: i128 = 500;
    let received_payload: Bytes = BytesN::<20>::random(&env).into();

    let source_address_bytes = Bytes::from_slice(&env, source_address.to_string().as_bytes());

    // Execute token receive message
    destination_app.execute_with_interchain_token(
        &source_chain,
        &String::from_str(&env, "test_message"),
        &source_address_bytes,
        &received_payload,
        &received_token_id,
        &received_token_address,
        &received_amount,
    );
}
