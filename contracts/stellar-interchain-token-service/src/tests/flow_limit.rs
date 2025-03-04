use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String};
use stellar_axelar_gas_service::testutils::setup_gas_token;
use stellar_axelar_gateway::testutils::{approve_gateway_messages, TestSignerSet};
use stellar_axelar_gateway::types::Message as GatewayMessage;
use stellar_axelar_gateway::AxelarGatewayClient;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::traits::BytesExt;
use stellar_axelar_std::{assert_auth, assert_contract_err, events};

use super::utils::setup_env;
use crate::error::ContractError;
use crate::event::FlowLimitSetEvent;
use crate::testutils::setup_its_token;
use crate::types::{HubMessage, InterchainTransfer, Message};
use crate::InterchainTokenServiceClient;

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

    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let supply = i128::MAX;
    let deployer = Address::generate(&env);
    let (token_id, _) = setup_its_token(&env, &client, &deployer, supply);

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
    let sender = Address::generate(env).to_string_bytes();
    let recipient = Address::generate(env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();

    let msg = HubMessage::ReceiveFromHub {
        source_chain: source_chain.clone(),
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

fn execute_its_transfer(
    env: &Env,
    client: &InterchainTokenServiceClient,
    gateway: &GatewayConfig,
    token_id: &BytesN<32>,
    amount: i128,
) {
    let msg = approve_its_transfer(env, client, gateway, token_id, amount);

    client.execute(
        &msg.source_chain,
        &msg.message_id,
        &msg.source_address,
        &msg.payload,
    )
}

#[test]
fn set_flow_limit_succeeds() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);

    assert_eq!(client.flow_limit(&token_id), None);

    assert_auth!(
        client.operator(),
        client.set_flow_limit(&token_id, &Some(dummy_flow_limit()))
    );
    goldie::assert!(events::fmt_last_emitted_event::<FlowLimitSetEvent>(&env));

    assert_eq!(client.flow_limit(&token_id), Some(dummy_flow_limit()));
}

#[test]
fn set_flow_limit_to_none_succeeds() {
    let (env, client, _, token) = setup();

    assert_eq!(client.flow_limit(&token.id), Some(dummy_flow_limit()));

    assert_auth!(
        client.operator(),
        client.set_flow_limit(&token.id, &None::<i128>)
    );
    goldie::assert!(events::fmt_last_emitted_event::<FlowLimitSetEvent>(&env));

    assert_eq!(client.flow_limit(&token.id), None);
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
            &Some(gas_token),
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

    execute_its_transfer(&env, &client, &gateway, &token.id, amount);

    assert_eq!(client.flow_in_amount(&token.id), amount);

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + EPOCH_TIME);

    assert_eq!(client.flow_in_amount(&token.id), 0);
}

#[test]
fn add_flow_in_succeeds() {
    let (env, client, gateway, token) = setup();

    let amount = dummy_flow_limit();

    assert_eq!(client.flow_in_amount(&token.id), 0);

    execute_its_transfer(&env, &client, &gateway, &token.id, amount);

    assert_eq!(client.flow_in_amount(&token.id), amount);
    assert_eq!(client.flow_out_amount(&token.id), 0);
}

#[test]
fn add_flow_in_fails_on_exceeding_flow_limit() {
    let (env, client, gateway, token) = setup();

    let amount = dummy_flow_limit();

    execute_its_transfer(&env, &client, &gateway, &token.id, amount);

    assert_eq!(client.flow_in_amount(&token.id), amount);

    let amount = 1;
    let msg = approve_its_transfer(&env, &client, &gateway, &token.id, amount);

    assert_contract_err!(
        client.try_execute(
            &msg.source_chain,
            &msg.message_id,
            &msg.source_address,
            &msg.payload
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
        &Some(gas_token),
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
        &Some(gas_token.clone()),
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
            &Some(gas_token),
        ),
        ContractError::FlowLimitExceeded
    );
}

#[test]
fn add_flow_fails_on_flow_comparison_overflow() {
    let cases = std::vec![
        (i128::MAX - 50, i128::MAX - 51, 2),
        (i128::MAX - 100, i128::MAX - 101, 2),
        (i128::MAX / 2 + 1, i128::MAX / 2 + 1, 2),
    ];

    for (flow_limit, flow_in, flow_out) in &cases {
        let (env, client, gateway, token) = setup();
        let gas_token = setup_gas_token(&env, &token.deployer);

        client
            .mock_all_auths()
            .set_flow_limit(&token.id, &Some(*flow_limit));

        let (destination_chain, destination_address, data) = dummy_transfer_params(&env);
        client
            .mock_all_auths()
            .set_trusted_chain(&destination_chain);

        execute_its_transfer(&env, &client, &gateway, &token.id, *flow_in);

        assert_contract_err!(
            client.mock_all_auths().try_interchain_transfer(
                &token.deployer,
                &token.id,
                &destination_chain,
                &destination_address,
                flow_out,
                &data,
                &Some(gas_token)
            ),
            ContractError::FlowAmountOverflow
        );
    }

    for (flow_limit, flow_out, flow_in) in cases {
        let (env, client, gateway, token) = setup();
        let gas_token = setup_gas_token(&env, &token.deployer);

        client
            .mock_all_auths()
            .set_flow_limit(&token.id, &Some(flow_limit));

        let (destination_chain, destination_address, data) = dummy_transfer_params(&env);
        client
            .mock_all_auths()
            .set_trusted_chain(&destination_chain);

        client.mock_all_auths().interchain_transfer(
            &token.deployer,
            &token.id,
            &destination_chain,
            &destination_address,
            &flow_out,
            &data,
            &Some(gas_token),
        );

        let msg = approve_its_transfer(&env, &client, &gateway, &token.id, flow_in);

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
