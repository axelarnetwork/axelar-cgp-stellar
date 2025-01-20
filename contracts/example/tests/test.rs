#![cfg(test)]
extern crate std;

use example::{Example, ExampleClient};
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::token::{self, StellarAssetClient};
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
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::traits::BytesExt;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{auth_invocation, events};
use stellar_interchain_token_service::{InterchainTokenService, InterchainTokenServiceClient};

const INTERCHAIN_TOKEN_WASM_HASH: &[u8] = include_bytes!("./testdata/interchain_token.wasm");
const ITS_HUB_ADDRESS: &str = "hub_address";
const SOURCE_CHAIN_NAME: &str = "source";
const DESTINATION_CHAIN_NAME: &str = "destination";

fn setup_gateway<'a>(env: &Env) -> (TestSignerSet, AxelarGatewayClient<'a>) {
    let (signers, client) = testutils::setup_gateway(env, 0, 5);
    (signers, client)
}

fn setup_gas_service<'a>(env: &Env) -> (AxelarGasServiceClient<'a>, Address, Address) {
    let owner: Address = Address::generate(env);
    let gas_collector: Address = Address::generate(env);
    let gas_service_address = env.register(AxelarGasService, (&owner, &gas_collector));
    let gas_service_client = AxelarGasServiceClient::new(env, &gas_service_address);

    (gas_service_client, gas_collector, gas_service_address)
}

fn setup_app<'a>(
    env: &Env,
    gateway: &Address,
    gas_service: &Address,
    its: &Address,
) -> ExampleClient<'a> {
    let id = env.register(Example, (gateway, gas_service, its));
    let client = ExampleClient::new(env, &id);

    client
}

fn setup_its<'a>(
    env: &Env,
    gateway: &Address,
    gas_service: &Address,
    chain_name: &String,
) -> (InterchainTokenServiceClient<'a>, Address) {
    let owner = Address::generate(env);
    let operator = Address::generate(env);
    let its_hub_address = String::from_str(env, ITS_HUB_ADDRESS);
    let native_token_address = Address::generate(env);
    let interchain_token_wasm_hash = env
        .deployer()
        .upload_contract_wasm(INTERCHAIN_TOKEN_WASM_HASH);

    let its_address = env.register(
        InterchainTokenService,
        (
            &owner,
            &operator,
            gateway,
            gas_service,
            its_hub_address,
            chain_name.clone(),
            native_token_address,
            interchain_token_wasm_hash,
        ),
    );

    let its_client = InterchainTokenServiceClient::new(env, &its_address);

    (its_client, its_address)
}

fn setup_its_token(
    env: &Env,
    client: &InterchainTokenServiceClient,
    sender: &Address,
    supply: i128,
) -> (BytesN<32>, Address) {
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

    let token_address = client.token_address(&token_id);

    (token_id, token_address)
}

#[test]
fn gmp_example() {
    let env = Env::default();

    let user: Address = Address::generate(&env);

    // Setup source Axelar gateway
    let source_chain = String::from_str(&env, SOURCE_CHAIN_NAME);
    let (_, source_gateway_client) = setup_gateway(&env);
    let source_gateway_address = source_gateway_client.address;
    let (source_gas_service_client, _source_gas_collector, source_gas_service_address) =
        setup_gas_service(&env);
    let (_, source_its_address) = setup_its(
        &env,
        &source_gateway_address,
        &source_gas_service_address,
        &source_chain,
    );
    let source_app = setup_app(
        &env,
        &source_gateway_address,
        &source_gas_service_address,
        &source_its_address,
    );

    // Setup destination Axelar gateway
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN_NAME);
    let (signers, destination_gateway_client) = setup_gateway(&env);
    let (
        _destination_gas_service_client,
        _destination_gas_collector,
        destination_gas_service_address,
    ) = setup_gas_service(&env);
    let (_, destination_its_address) = setup_its(
        &env,
        &destination_gateway_client.address,
        &destination_gas_service_address,
        &destination_chain,
    );
    let destination_app = setup_app(
        &env,
        &destination_gateway_client.address,
        &destination_gas_service_address,
        &destination_its_address,
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

    asset_client.mock_all_auths().mint(&user, &gas_amount);

    source_app.mock_all_auths().send(
        &user,
        &destination_chain,
        &destination_address,
        &payload,
        &gas_token,
    );

    let transfer_auth = auth_invocation!(
        &env,
        user,
        asset_client.transfer(&user, source_gas_service_address, gas_token.amount)
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
            destination_address.clone(),
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
}

#[test]
fn its_example() {
    let env = Env::default();

    let user = Address::generate(&env).to_string_bytes();

    let (signers, gateway_client) = setup_gateway(&env);
    let (gas_service_client, _, gas_service_address) = setup_gas_service(&env);
    let chain_name = String::from_str(&env, "chain_name");
    let (source_its_client, _) = setup_its(
        &env,
        &gateway_client.address,
        &gas_service_address,
        &chain_name,
    );
    let source_chain = source_its_client.its_hub_chain_name();
    let source_address: String = source_its_client.its_hub_address();

    let amount = 1000;
    let deployer = Address::generate(&env);
    let (token_id, _) = setup_its_token(&env, &source_its_client, &deployer, amount);

    let example_app_address = env.register(
        Example,
        (
            &gateway_client.address,
            &gas_service_client.address,
            &source_its_client.address,
        ),
    );
    let destination_address = example_app_address.to_string_bytes();
    let original_source_chain = String::from_str(&env, "ethereum");
    source_its_client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let data = Bytes::from_hex(&env, "dead");

    let msg = stellar_interchain_token_service::types::HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
        message: stellar_interchain_token_service::types::Message::InterchainTransfer(
            stellar_interchain_token_service::types::InterchainTransfer {
                token_id: token_id.clone(),
                source_address: user,
                destination_address,
                amount,
                data: Some(data.clone()),
            },
        ),
    };
    let payload = msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    let message_id = String::from_str(&env, "test");

    let messages = vec![
        &env,
        Message {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
            source_address: source_address.clone(),
            contract_address: source_its_client.address.clone(),
            payload_hash,
        },
    ];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    gateway_client.approve_messages(&messages, &proof);

    source_its_client.execute(&source_chain, &message_id, &source_address, &payload);

    let token = token::TokenClient::new(&env, &source_its_client.token_address(&token_id));
    assert_eq!(token.balance(&example_app_address), amount);
}
