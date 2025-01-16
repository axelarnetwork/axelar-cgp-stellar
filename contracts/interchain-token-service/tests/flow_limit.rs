mod utils;

use axelar_gateway::testutils::TestSignerSet;
use axelar_gateway::types::Message as GatewayMessage;
use axelar_gateway::AxelarGatewayClient;
use axelar_soroban_std::traits::BytesExt;
use axelar_soroban_std::{assert_contract_err, assert_invoke_auth_ok, events};
use interchain_token_service::error::ContractError;
use interchain_token_service::event::FlowLimitSetEvent;
use interchain_token_service::types::{HubMessage, InterchainTransfer, Message};
use interchain_token_service::InterchainTokenServiceClient;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String};
use utils::{
    approve_gateway_messages, register_chains, setup_env, setup_gas_token, setup_its_token,
    HUB_CHAIN,
};

struct GatewayConfig<'a> {
    client: AxelarGatewayClient<'a>,
    signers: TestSignerSet,
}

struct TokenConfig {
    id: BytesN<32>,
    deployer: Address,
}

struct ApprovedMessage {
    source_chain: String,
    message_id: String,
    source_address: String,
    payload: Bytes,
}

const EPOCH_TIME: u64 = 6 * 60 * 60;

const fn dummy_flow_limit() -> i128 {
    1000
}

fn dummy_transfer_params(env: &Env) -> (String, Bytes, Option<Bytes>) {
    let destination_chain = String::from_str(env, "ethereum");
    let destination_address = Bytes::from_hex(env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = None;

    (destination_chain, destination_address, data)
}

fn setup<'a>() -> (
    Env,
    InterchainTokenServiceClient<'a>,
    GatewayConfig<'a>,
    TokenConfig,
) {
    let (env, client, gateway_client, _, signers) = setup_env();

    register_chains(&env, &client);

    let supply = i128::MAX;
    let deployer = Address::generate(&env);
    let token_id = setup_its_token(&env, &client, &deployer, supply);

    client
        .mock_all_auths()
        .set_flow_limit(&token_id, &Some(dummy_flow_limit()));

    (
        env,
        client,
        GatewayConfig {
            client: gateway_client,
            signers,
        },
        TokenConfig {
            id: token_id,
            deployer,
        },
    )
}

fn approve_its_transfer(
    env: &Env,
    client: &InterchainTokenServiceClient,
    gateway: &GatewayConfig,
    token_id: &BytesN<32>,
    amount: i128,
) -> ApprovedMessage {
    let sender = Address::generate(env).to_xdr(env);
    let recipient = Address::generate(env).to_xdr(env);
    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(env).to_string();

    let msg = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(env, HUB_CHAIN),
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id: token_id.clone(),
            source_address: sender,
            destination_address: recipient,
            amount,
            data: None,
        }),
    };
    let payload = msg.abi_encode(env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    let message_id = Address::generate(env).to_string();

    let messages = vec![
        &env,
        GatewayMessage {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
            source_address: source_address.clone(),
            contract_address: client.address.clone(),
            payload_hash,
        },
    ];

    approve_gateway_messages(env, &gateway.client, gateway.signers.clone(), messages);

    ApprovedMessage {
        source_chain,
        message_id,
        source_address,
        payload,
    }
}

#[test]
fn set_flow_limit_succeeds() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);

    assert_eq!(client.flow_limit(&token_id), None);

    assert_invoke_auth_ok!(
        client.operator(),
        client.try_set_flow_limit(&token_id, &Some(dummy_flow_limit()))
    );

    assert_eq!(client.flow_limit(&token_id), Some(dummy_flow_limit()));

    goldie::assert!(events::fmt_last_emitted_event::<FlowLimitSetEvent>(&env));
}

#[test]
fn zero_flow_limit_effectively_freezes_token() {
    let (env, client, gateway, token) = setup();
    let gas_token = setup_gas_token(&env, &token.deployer);

    client.mock_all_auths().set_flow_limit(&token.id, &Some(0));

    let amount = 1;
    let msg = approve_its_transfer(&env, &client, &gateway, &token.id, amount);

    assert_contract_err!(
        client.try_execute(
            &msg.source_chain,
            &msg.message_id,
            &msg.source_address,
            &msg.payload,
        ),
        ContractError::FlowLimitExceeded
    );

    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &token.deployer,
            &token.id,
            &destination_chain,
            &destination_address,
            &amount,
            &data,
            &gas_token,
        ),
        ContractError::FlowLimitExceeded
    );
}

#[test]
fn set_flow_limit_fails_on_negative_limit() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);

    let invalid_limit = Some(-1);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_set_flow_limit(&token_id, &invalid_limit),
        ContractError::InvalidFlowLimit
    );
}

#[test]
fn flow_limit_resets_after_epoch() {
    let (env, client, gateway, token) = setup();

    let amount = dummy_flow_limit();
    let msg = approve_its_transfer(&env, &client, &gateway, &token.id, amount);

    client.execute(
        &msg.source_chain,
        &msg.message_id,
        &msg.source_address,
        &msg.payload,
    );

    assert_eq!(client.flow_in_amount(&token.id), amount);

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + EPOCH_TIME);
    assert_eq!(client.flow_in_amount(&token.id), 0);
}

#[test]
fn add_flow_in_succeeds() {
    let (env, client, gateway, token) = setup();

    let amount = dummy_flow_limit();
    let msg = approve_its_transfer(&env, &client, &gateway, &token.id, amount);

    assert_eq!(client.flow_in_amount(&token.id), 0);

    client.execute(
        &msg.source_chain,
        &msg.message_id,
        &msg.source_address,
        &msg.payload,
    );

    assert_eq!(client.flow_in_amount(&token.id), amount);
    assert_eq!(client.flow_out_amount(&token.id), 0);
}

#[test]
fn add_flow_in_fails_on_exceeding_flow_limit() {
    let (env, client, gateway, token) = setup();

    let amount = dummy_flow_limit();
    let msg = approve_its_transfer(&env, &client, &gateway, &token.id, amount);

    client.execute(
        &msg.source_chain,
        &msg.message_id,
        &msg.source_address,
        &msg.payload,
    );

    assert_eq!(client.flow_in_amount(&token.id), amount);

    let second_amount = 1;
    let second_msg = approve_its_transfer(&env, &client, &gateway, &token.id, second_amount);

    assert_contract_err!(
        client.try_execute(
            &second_msg.source_chain,
            &second_msg.message_id,
            &second_msg.source_address,
            &second_msg.payload
        ),
        ContractError::FlowLimitExceeded
    );
}

#[test]
fn add_flow_out_succeeds() {
    let (env, client, _, token) = setup();
    let gas_token = setup_gas_token(&env, &token.deployer);

    let amount = dummy_flow_limit();
    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    assert_eq!(client.flow_out_amount(&token.id), 0);

    client.mock_all_auths().interchain_transfer(
        &token.deployer,
        &token.id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &gas_token,
    );

    assert_eq!(client.flow_out_amount(&token.id), amount);
    assert_eq!(client.flow_in_amount(&token.id), 0);
}

#[test]
fn add_flow_out_fails_on_exceeding_flow_limit() {
    let (env, client, _, token) = setup();
    let gas_token = setup_gas_token(&env, &token.deployer);

    let amount = dummy_flow_limit();
    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &token.deployer,
        &token.id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &gas_token,
    );

    assert_eq!(client.flow_out_amount(&token.id), amount);

    let second_amount = 1;

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &token.deployer,
            &token.id,
            &destination_chain,
            &destination_address,
            &second_amount,
            &data,
            &gas_token,
        ),
        ContractError::FlowLimitExceeded
    );
}

const LARGE_FLOW_LIMIT: i128 = i128::MAX - 50;
const OVERFLOW_CASES: &[(i128, i128)] = &[
    (LARGE_FLOW_LIMIT - 1, 2),
    (i128::MAX - 100, 2),
    (LARGE_FLOW_LIMIT, 1),
];

enum OverflowTestDirection {
    InThenOut,
    OutThenIn,
}

fn test_overflow_case(case: &(i128, i128), direction: OverflowTestDirection) {
    let (env, client, gateway, token) = setup();
    let gas_token = setup_gas_token(&env, &token.deployer);

    client
        .mock_all_auths()
        .set_flow_limit(&token.id, &Some(LARGE_FLOW_LIMIT));

    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);
    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    match direction {
        OverflowTestDirection::InThenOut => {
            let amount_in = case.0;
            let amount_out = case.1;

            let msg = approve_its_transfer(&env, &client, &gateway, &token.id, amount_in);

            client.execute(
                &msg.source_chain,
                &msg.message_id,
                &msg.source_address,
                &msg.payload,
            );

            assert_contract_err!(
                client.mock_all_auths().try_interchain_transfer(
                    &token.deployer,
                    &token.id,
                    &destination_chain,
                    &destination_address,
                    &amount_out,
                    &data,
                    &gas_token
                ),
                ContractError::FlowAmountOverflow
            );
        }
        OverflowTestDirection::OutThenIn => {
            let amount_out = case.0;
            let amount_in = case.1;

            client.mock_all_auths().interchain_transfer(
                &token.deployer,
                &token.id,
                &destination_chain,
                &destination_address,
                &amount_out,
                &data,
                &gas_token,
            );

            let msg = approve_its_transfer(&env, &client, &gateway, &token.id, amount_in);

            assert_contract_err!(
                client.try_execute(
                    &msg.source_chain,
                    &msg.message_id,
                    &msg.source_address,
                    &msg.payload
                ),
                ContractError::FlowAmountOverflow
            );
        }
    }
}

#[test]
fn add_flow_fails_on_flow_comparison_overflow() {
    for case in OVERFLOW_CASES {
        test_overflow_case(case, OverflowTestDirection::InThenOut);
        test_overflow_case(case, OverflowTestDirection::OutThenIn);
    }
}
