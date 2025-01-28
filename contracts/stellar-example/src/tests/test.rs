use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::token::{self, StellarAssetClient};
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, IntoVal, String, Symbol};
use stellar_axelar_gas_service::testutils::setup_gas_service;
use stellar_axelar_gas_service::AxelarGasServiceClient;
use stellar_axelar_gateway::event::{ContractCalledEvent, MessageApprovedEvent};
use stellar_axelar_gateway::testutils::{
    generate_proof, get_approve_hash, setup_gateway, TestSignerSet,
};
use stellar_axelar_gateway::types::Message;
use stellar_axelar_gateway::AxelarGatewayClient;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::traits::BytesExt;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{assert_contract_err, auth_invocation, events};
use stellar_interchain_token_service::event::TrustedChainSetEvent;
use stellar_interchain_token_service::testutils::{setup_its, setup_its_token};
use stellar_interchain_token_service::InterchainTokenServiceClient;

use crate::contract::ExampleError;
use crate::event::{ExecutedEvent, TokenReceivedEvent};
use crate::{Example, ExampleClient};

const SOURCE_CHAIN_NAME: &str = "source";
const DESTINATION_CHAIN_NAME: &str = "destination";

struct TestConfig<'a> {
    signers: TestSignerSet,
    gateway_client: AxelarGatewayClient<'a>,
    gas_service_client: AxelarGasServiceClient<'a>,
    its_client: InterchainTokenServiceClient<'a>,
    app: ExampleClient<'a>,
}

fn setup_app<'a>(env: &Env, chain_name: Option<String>) -> TestConfig<'a> {
    let (signers, gateway_client) = setup_gateway(env, 0, 5);
    let gas_service_client = setup_gas_service(env);
    let its_client = setup_its(env, &gateway_client, &gas_service_client, chain_name);
    let app = env.register(
        Example,
        (
            &gateway_client.address,
            &gas_service_client.address,
            &its_client.address,
        ),
    );
    let app = ExampleClient::new(env, &app);

    TestConfig {
        signers,
        gateway_client,
        gas_service_client,
        its_client,
        app,
    }
}

#[test]
fn gmp_example() {
    let env = Env::default();

    let user: Address = Address::generate(&env);

    // Setup source Axelar gateway
    let source_chain = String::from_str(&env, SOURCE_CHAIN_NAME);
    let TestConfig {
        gas_service_client: source_gas_service_client,
        app: source_app,
        ..
    } = setup_app(&env, None);

    // Setup destination Axelar gateway
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN_NAME);
    let TestConfig {
        signers: destination_signers,
        gateway_client: destination_gateway_client,
        app: destination_app,
        ..
    } = setup_app(&env, None);

    // Set cross-chain message params
    let source_address = source_app.address.to_string();
    let destination_address = destination_app.address.to_string();
    let payload = Bytes::from_hex(&env, "dead");
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
        asset_client.transfer(&user, &source_gas_service_client.address, gas_token.amount)
    );

    let source_gas_service_client = source_gas_service_client;

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

    let source_app = source_app;

    let send_auth = auth_invocation!(
        &env,
        user,
        source_app.send(
            &user,
            destination_chain,
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
    let contract_call_event = events::fmt_last_emitted_event::<ContractCalledEvent>(&env);

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
    let proof = generate_proof(&env, data_hash, destination_signers);

    // Submitting signed message approval to destination Axelar gateway
    destination_gateway_client.approve_messages(&messages, &proof);

    let message_approved_event = events::fmt_last_emitted_event::<MessageApprovedEvent>(&env);

    // Executing message on destination app
    destination_app.execute(&source_chain, &message_id, &source_address, &payload);

    let executed_event = events::fmt_last_emitted_event::<ExecutedEvent>(&env);

    goldie::assert!([contract_call_event, message_approved_event, executed_event].join("\n\n"));
}

#[test]
fn its_example() {
    let env = Env::default();

    let user = Address::generate(&env);

    let original_source_chain = String::from_str(&env, SOURCE_CHAIN_NAME);
    let TestConfig {
        its_client: original_source_its_client,
        app: original_source_app,
        ..
    } = setup_app(&env, Some(original_source_chain.clone()));

    let hub_source_chain = original_source_its_client.its_hub_chain_name();
    let hub_source_address = original_source_its_client.its_hub_address();

    let destination_chain = String::from_str(&env, DESTINATION_CHAIN_NAME);
    let TestConfig {
        signers: destination_signers,
        gateway_client: destination_gateway_client,
        its_client: destination_its_client,
        app: destination_app,
        ..
    } = setup_app(&env, Some(destination_chain.clone()));

    let transfer_amount = 1000;

    let (token_id, token_metadata) =
        setup_its_token(&env, &original_source_its_client, &user, transfer_amount);
    let token_address = original_source_its_client.token_address(&token_id);

    let gas_token_contract = env.register_stellar_asset_contract_v2(user.clone());
    let gas_amount = 50;
    let gas_token = Token {
        address: gas_token_contract.address(),
        amount: gas_amount,
    };
    StellarAssetClient::new(&env, &gas_token.address)
        .mock_all_auths()
        .mint(&user, &gas_token.amount);

    original_source_its_client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    let original_source_trusted_chain_set_event =
        events::fmt_last_emitted_event::<TrustedChainSetEvent>(&env);

    destination_its_client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let destination_trusted_chain_set_event =
        events::fmt_last_emitted_event::<TrustedChainSetEvent>(&env);

    let data = Address::generate(&env).to_string_bytes();

    original_source_its_client
        .mock_all_auths()
        .register_canonical_token(&token_address);

    original_source_its_client
        .mock_all_auths()
        .deploy_remote_canonical_token(&token_address, &destination_chain, &user, &gas_token);

    let token_asset_client = StellarAssetClient::new(&env, &gas_token.address);
    token_asset_client
        .mock_all_auths()
        .mint(&user, &transfer_amount);

    // DeployInterchainToken execute
    let deploy_msg = stellar_interchain_token_service::types::HubMessage::ReceiveFromHub {
        source_chain: original_source_chain.clone(),
        message: stellar_interchain_token_service::types::Message::DeployInterchainToken(
            stellar_interchain_token_service::types::DeployInterchainToken {
                token_id: token_id.clone(),
                name: token_metadata.name.clone(),
                symbol: token_metadata.symbol.clone(),
                decimals: token_metadata.decimal as u8,
                minter: None,
            },
        ),
    };
    let payload = deploy_msg.abi_encode(&env).unwrap();

    let message_id = String::from_str(&env, "deploy-message-id");

    let deploy_messages = vec![
        &env,
        Message {
            source_chain: hub_source_chain.clone(),
            message_id: message_id.clone(),
            source_address: hub_source_address.clone(),
            contract_address: destination_its_client.address.clone(),
            payload_hash: env.crypto().keccak256(&payload).into(),
        },
    ];

    let proof = generate_proof(
        &env,
        get_approve_hash(&env, deploy_messages.clone()),
        destination_signers.clone(),
    );

    destination_gateway_client.approve_messages(&deploy_messages, &proof);

    destination_its_client.execute(
        &hub_source_chain,
        &message_id,
        &hub_source_address,
        &payload,
    );
    // END DeployInterchainToken execute

    original_source_app.mock_all_auths().send_token(
        &user,
        &token_id,
        &destination_chain,
        &destination_app.address.to_string_bytes(),
        &transfer_amount,
        &Some(data.clone()),
        &gas_token,
    );

    // InterchainTransfer execute
    let transfer_msg = stellar_interchain_token_service::types::HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
        message: stellar_interchain_token_service::types::Message::InterchainTransfer(
            stellar_interchain_token_service::types::InterchainTransfer {
                token_id: token_id.clone(),
                source_address: user.to_string_bytes(),
                destination_address: destination_app.address.to_string_bytes(),
                amount: transfer_amount,
                data: Some(data.clone()),
            },
        ),
    };
    let payload = transfer_msg.abi_encode(&env).unwrap();

    let message_id = String::from_str(&env, "transfer-message-id");

    let transfer_messages = vec![
        &env,
        Message {
            source_chain: hub_source_chain.clone(),
            message_id: message_id.clone(),
            source_address: hub_source_address.clone(),
            contract_address: destination_its_client.address.clone(),
            payload_hash: env.crypto().keccak256(&payload).into(),
        },
    ];

    let proof = generate_proof(
        &env,
        get_approve_hash(&env, transfer_messages.clone()),
        destination_signers.clone(),
    );

    destination_gateway_client.approve_messages(&transfer_messages, &proof);

    let message_approved_event = events::fmt_last_emitted_event::<MessageApprovedEvent>(&env);

    destination_its_client.execute(
        &hub_source_chain,
        &message_id,
        &hub_source_address,
        &payload,
    );
    // END InterchainTransfer execute

    let token_received_event = events::fmt_last_emitted_event::<TokenReceivedEvent>(&env);

    goldie::assert!([
        destination_trusted_chain_set_event,
        original_source_trusted_chain_set_event,
        message_approved_event,
        token_received_event
    ]
    .join("\n\n"));

    let destination_token_client =
        token::TokenClient::new(&env, &destination_its_client.token_address(&token_id));
    assert_eq!(
        destination_token_client.balance(&destination_app.address),
        0
    );

    let recipient = Address::from_string_bytes(&data);

    assert_eq!(
        destination_token_client.balance(&destination_app.address),
        0
    );
    assert_eq!(
        destination_token_client.balance(&recipient),
        transfer_amount
    );
}

#[test]
fn constructor_succeeds() {
    let env = Env::default();

    let TestConfig {
        gateway_client,
        gas_service_client,
        its_client,
        ..
    } = setup_app(&env, None);

    let contract_id = env.register(
        Example,
        (
            &gateway_client.address,
            &gas_service_client.address,
            &its_client.address,
        ),
    );
    let client = ExampleClient::new(&env, &contract_id);

    assert_eq!(client.gateway(), gateway_client.address);
    assert_eq!(client.gas_service(), gas_service_client.address);
    assert_eq!(client.interchain_token_service(), its_client.address);
}

#[test]
fn execute_fails_with_not_approved() {
    let env = Env::default();

    let TestConfig { app, .. } = setup_app(&env, None);

    let source_chain = String::from_str(&env, "ethereum");
    let message_id = String::from_str(&env, "test");
    let source_address = String::from_str(&env, "0x123");
    let payload = Bytes::from_array(&env, &[1u8; 32]);

    assert_contract_err!(
        app.try_execute(&source_chain, &message_id, &source_address, &payload),
        ExampleError::NotApproved
    );
}
