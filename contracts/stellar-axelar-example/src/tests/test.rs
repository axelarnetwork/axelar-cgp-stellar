use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, StellarAssetContract,
};
use soroban_sdk::token::{self, StellarAssetClient};
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, IntoVal, String, Symbol};
use stellar_axelar_gas_service::testutils::{setup_gas_service, setup_gas_token};
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
use stellar_axelar_std::{assert_contract_err, assert_ok, auth_invocation, events};
use stellar_interchain_token_service::testutils::setup_its;
use stellar_interchain_token_service::InterchainTokenServiceClient;

use crate::contract::AxelarExampleError;
use crate::event::{ExecutedEvent, TokenReceivedEvent, TokenSentEvent};
use crate::{AxelarExample, AxelarExampleClient};

const SOURCE_CHAIN_NAME: &str = "source";
const DESTINATION_CHAIN_NAME: &str = "destination";

struct TestConfig<'a> {
    signers: TestSignerSet,
    gateway: AxelarGatewayClient<'a>,
    gas_service: AxelarGasServiceClient<'a>,
    its: InterchainTokenServiceClient<'a>,
    app: AxelarExampleClient<'a>,
}

fn setup_app<'a>(env: &Env, chain_name: String) -> TestConfig<'a> {
    let (signers, gateway) = setup_gateway(env, 0, 5);
    let gas_service = setup_gas_service(env);
    let its = setup_its(env, &gateway, &gas_service, Some(chain_name));
    let app = env.register(
        AxelarExample,
        (&gateway.address, &gas_service.address, &its.address),
    );
    let app = AxelarExampleClient::new(env, &app);

    TestConfig {
        signers,
        gateway,
        gas_service,
        its,
        app,
    }
}

fn setup_tokens(env: &Env, user: &Address, amount: i128) -> (StellarAssetContract, Token) {
    let token = env.register_stellar_asset_contract_v2(user.clone());
    StellarAssetClient::new(env, &token.address())
        .mock_all_auths()
        .mint(user, &amount);

    let gas_token = setup_gas_token(env, user);

    StellarAssetClient::new(env, &gas_token.address)
        .mock_all_auths()
        .mint(user, &1);

    (token, gas_token)
}

#[test]
fn gmp_example() {
    let env = Env::default();

    let user: Address = Address::generate(&env);

    // Setup source Axelar gateway
    let source_chain = String::from_str(&env, SOURCE_CHAIN_NAME);
    let TestConfig {
        gas_service: source_gas_service,
        app: source_app,
        ..
    } = setup_app(&env, source_chain.clone());

    // Setup destination Axelar gateway
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN_NAME);
    let TestConfig {
        signers: destination_signers,
        gateway: destination_gateway_client,
        app: destination_app,
        ..
    } = setup_app(&env, destination_chain.clone());

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
        &Some(gas_token.clone()),
    );

    let transfer_auth = auth_invocation!(
        &env,
        user,
        asset_client.transfer(&user, &source_gas_service.address, gas_token.amount)
    );

    let source_gas_service_client = source_gas_service;

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

    // Setup source app
    let source_chain = String::from_str(&env, SOURCE_CHAIN_NAME);
    let TestConfig {
        its: source_its,
        app: source_app,
        ..
    } = setup_app(&env, source_chain.clone());

    let hub_chain = source_its.its_hub_chain_name();
    let hub_address = source_its.its_hub_address();

    // Setup destination app
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN_NAME);
    let TestConfig {
        signers: destination_signers,
        gateway: destination_gateway,
        its: destination_its,
        app: destination_app,
        ..
    } = setup_app(&env, destination_chain.clone());

    let transfer_amount = 1000;

    // Setup tokens
    let (token, gas_token) = setup_tokens(&env, &user, transfer_amount);

    source_its
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    destination_its
        .mock_all_auths()
        .set_trusted_chain(&source_chain);

    let recipient = Address::generate(&env);

    // Register and deploy tokens on ITS
    source_its
        .mock_all_auths()
        .register_canonical_token(&token.address());

    let token_id = source_its.mock_all_auths().deploy_remote_canonical_token(
        &token.address(),
        &destination_chain,
        &user,
        &Some(gas_token.clone()),
    );

    // Execute DeployInterchainToken message on destination
    let deploy_msg_payload = assert_ok!(
        stellar_interchain_token_service::types::HubMessage::ReceiveFromHub {
            source_chain: source_chain.clone(),
            message: stellar_interchain_token_service::types::Message::DeployInterchainToken(
                stellar_interchain_token_service::types::DeployInterchainToken {
                    token_id: token_id.clone(),
                    name: String::from_str(&env, "Test"),
                    symbol: String::from_str(&env, "TEST"),
                    decimals: 18,
                    minter: None,
                },
            ),
        }
        .abi_encode(&env)
    );

    let message_id = String::from_str(&env, "deploy-message-id");

    let deploy_messages = vec![
        &env,
        Message {
            source_chain: hub_chain.clone(),
            message_id: message_id.clone(),
            source_address: hub_address.clone(),
            contract_address: destination_its.address.clone(),
            payload_hash: env.crypto().keccak256(&deploy_msg_payload).into(),
        },
    ];

    let proof = generate_proof(
        &env,
        get_approve_hash(&env, deploy_messages.clone()),
        destination_signers.clone(),
    );

    destination_gateway.approve_messages(&deploy_messages, &proof);

    destination_its.execute(&hub_chain, &message_id, &hub_address, &deploy_msg_payload);

    // Send tokens to destination app
    source_app.mock_all_auths().send_token(
        &user,
        &token_id,
        &destination_chain,
        &destination_app.address.to_string_bytes(),
        &transfer_amount,
        &Some(recipient.to_string_bytes()),
        &Some(gas_token),
    );

    let token_sent_event = events::fmt_last_emitted_event::<TokenSentEvent>(&env);

    // Execute InterchainTransfer message on destination
    let transfer_msg_payload = assert_ok!(
        stellar_interchain_token_service::types::HubMessage::ReceiveFromHub {
            source_chain,
            message: stellar_interchain_token_service::types::Message::InterchainTransfer(
                stellar_interchain_token_service::types::InterchainTransfer {
                    token_id: token_id.clone(),
                    source_address: user.to_string_bytes(),
                    destination_address: destination_app.address.to_string_bytes(),
                    amount: transfer_amount,
                    data: Some(recipient.to_string_bytes()),
                },
            ),
        }
        .abi_encode(&env)
    );

    let message_id = String::from_str(&env, "transfer-message-id");

    let transfer_messages = vec![
        &env,
        Message {
            source_chain: hub_chain.clone(),
            message_id: message_id.clone(),
            source_address: hub_address.clone(),
            contract_address: destination_its.address.clone(),
            payload_hash: env.crypto().keccak256(&transfer_msg_payload).into(),
        },
    ];

    let proof = generate_proof(
        &env,
        get_approve_hash(&env, transfer_messages.clone()),
        destination_signers,
    );

    destination_gateway.approve_messages(&transfer_messages, &proof);

    let message_approved_event = events::fmt_last_emitted_event::<MessageApprovedEvent>(&env);

    destination_its.execute(&hub_chain, &message_id, &hub_address, &transfer_msg_payload);

    let token_received_event = events::fmt_last_emitted_event::<TokenReceivedEvent>(&env);

    goldie::assert!([
        token_sent_event,
        message_approved_event,
        token_received_event
    ]
    .join("\n\n"));

    let destination_token =
        token::TokenClient::new(&env, &destination_its.registered_token_address(&token_id));
    assert_eq!(destination_token.balance(&destination_app.address), 0);

    let recipient = Address::from_string_bytes(&recipient.to_string_bytes());

    assert_eq!(destination_token.balance(&destination_app.address), 0);
    assert_eq!(destination_token.balance(&recipient), transfer_amount);
}

#[test]
fn constructor_succeeds() {
    let env = Env::default();

    let source_chain = String::from_str(&env, SOURCE_CHAIN_NAME);
    let TestConfig {
        gateway,
        gas_service,
        its,
        app,
        ..
    } = setup_app(&env, source_chain);

    assert_eq!(app.gateway(), gateway.address);
    assert_eq!(app.gas_service(), gas_service.address);
    assert_eq!(app.interchain_token_service(), its.address);
}

#[test]
fn execute_fails_with_not_approved() {
    let env = Env::default();

    let source_chain = String::from_str(&env, SOURCE_CHAIN_NAME);
    let TestConfig { app, .. } = setup_app(&env, source_chain);

    let source_chain = String::from_str(&env, "ethereum");
    let message_id = String::from_str(&env, "test");
    let source_address = String::from_str(&env, "0x123");
    let payload = Bytes::from_array(&env, &[1u8; 32]);

    assert_contract_err!(
        app.try_execute(&source_chain, &message_id, &source_address, &payload),
        AxelarExampleError::NotApproved
    );
}
