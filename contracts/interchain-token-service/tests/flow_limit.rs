mod utils;

use axelar_gateway::types::Message as GatewayMessage;
use axelar_soroban_std::{assert_auth, assert_contract_err, events, traits::BytesExt};
use interchain_token_service::{
    error::ContractError,
    event::FlowLimitSetEvent,
    types::{HubMessage, InterchainTransfer, Message},
    InterchainTokenServiceClient,
};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String, Vec};
use utils::{
    approve_gateway_messages, register_chains, setup_env, setup_gas_token, setup_its_token,
    HUB_CHAIN,
};

const TEST_FLOW_LIMIT: Option<i128> = Some(1000);
const EPOCH_TIME: u64 = 6 * 60 * 60;

fn setup_flow_limit(env: &Env, client: &InterchainTokenServiceClient) -> (BytesN<32>, Address) {
    let supply = i128::MAX;
    let deployer = Address::generate(env);
    let token_id = setup_its_token(env, client, &deployer, supply);

    client
        .mock_all_auths()
        .set_flow_limit(&token_id, &TEST_FLOW_LIMIT);

    (token_id, deployer)
}

fn create_interchain_transfer_message(
    env: &Env,
    client: &InterchainTokenServiceClient,
    token_id: &BytesN<32>,
    amount: i128,
) -> (String, String, String, Bytes, Vec<GatewayMessage>) {
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

    (source_chain, message_id, source_address, payload, messages)
}

#[test]
fn set_flow_limit_succeeds() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);

    assert_eq!(client.flow_limit(&token_id), None);

    assert_auth!(
        client.operator(),
        client.set_flow_limit(&token_id, &TEST_FLOW_LIMIT)
    );

    assert_eq!(client.flow_limit(&token_id), TEST_FLOW_LIMIT);

    goldie::assert!(events::fmt_last_emitted_event::<FlowLimitSetEvent>(&env));
}

#[test]
#[should_panic(expected = "Error(Contract, #20)")] // FlowLimitExceeded
fn zero_flow_limit_freezes_token() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);
    let (token_id, _) = setup_flow_limit(&env, &client);

    client.mock_all_auths().set_flow_limit(&token_id, &Some(0));

    let amount = 1;
    let (source_chain, message_id, source_address, payload, messages) =
        create_interchain_transfer_message(&env, &client, &token_id, amount);
    approve_gateway_messages(&env, &gateway_client, signers, messages);

    client.execute(&source_chain, &message_id, &source_address, &payload);
}

#[test]
fn set_flow_limit_fails_invalid_amount() {
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
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);
    let (token_id, _) = setup_flow_limit(&env, &client);

    let amount = TEST_FLOW_LIMIT.unwrap();

    let (source_chain, message_id, source_address, payload, messages) =
        create_interchain_transfer_message(&env, &client, &token_id, amount);
    approve_gateway_messages(&env, &gateway_client, signers.clone(), messages);
    client.execute(&source_chain, &message_id, &source_address, &payload);
    assert_eq!(client.flow_in_amount(&token_id), amount);

    let current_timestamp = env.ledger().timestamp();
    env.ledger().set_timestamp(current_timestamp + EPOCH_TIME);

    let (source_chain, message_id, source_address, payload, messages) =
        create_interchain_transfer_message(&env, &client, &token_id, amount);
    approve_gateway_messages(&env, &gateway_client, signers, messages);
    client.execute(&source_chain, &message_id, &source_address, &payload);
    assert_eq!(client.flow_in_amount(&token_id), amount);
}

#[test]
fn add_flow_in_succeeds() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);
    let (token_id, _) = setup_flow_limit(&env, &client);

    let amount = TEST_FLOW_LIMIT.unwrap();
    let (source_chain, message_id, source_address, payload, messages) =
        create_interchain_transfer_message(&env, &client, &token_id, amount);
    approve_gateway_messages(&env, &gateway_client, signers, messages);

    assert_eq!(client.flow_in_amount(&token_id), 0);

    client.execute(&source_chain, &message_id, &source_address, &payload);

    assert_eq!(client.flow_in_amount(&token_id), amount);
}

#[test]
#[should_panic(expected = "Error(Contract, #20)")] // ContractError::FlowLimitExceeded
fn add_flow_in_fails_exceeds_flow_limit() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);
    let (token_id, _) = setup_flow_limit(&env, &client);

    let amount = TEST_FLOW_LIMIT.unwrap();
    let (source_chain, message_id, source_address, payload, messages) =
        create_interchain_transfer_message(&env, &client, &token_id, amount);
    approve_gateway_messages(&env, &gateway_client, signers.clone(), messages);

    client.execute(&source_chain, &message_id, &source_address, &payload);

    let second_amount = 1;
    let (source_chain, message_id, source_address, payload, messages) =
        create_interchain_transfer_message(&env, &client, &token_id, second_amount);
    approve_gateway_messages(&env, &gateway_client, signers, messages);

    client.execute(&source_chain, &message_id, &source_address, &payload);
}

#[test]
fn add_flow_out_succeeds() {
    let (env, client, _, _, _) = setup_env();
    register_chains(&env, &client);
    let (token_id, sender) = setup_flow_limit(&env, &client);
    let gas_token = setup_gas_token(&env, &sender);

    let amount = 1000;
    let destination_chain = String::from_str(&env, "ethereum");
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = None;

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &gas_token,
    );

    assert_eq!(client.flow_out_amount(&token_id), amount);
}

#[test]
#[should_panic(expected = "Error(Contract, #20)")] // ContractError::FlowLimitExceeded
fn add_flow_out_fails_exceeds_flow_limit() {
    let (env, client, _, _, _) = setup_env();
    register_chains(&env, &client);
    let (token_id, sender) = setup_flow_limit(&env, &client);
    let gas_token = setup_gas_token(&env, &sender);

    let amount = TEST_FLOW_LIMIT.unwrap();
    let destination_chain = String::from_str(&env, "ethereum");
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = None;

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &gas_token,
    );

    let second_amount = 1;

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &second_amount,
        &data,
        &gas_token,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #21)")] // ContractError::FlowAmountOverflow
fn add_flow_fails_on_flow_comparison_overflow() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);
    let (token_id, sender) = setup_flow_limit(&env, &client);
    let gas_token = setup_gas_token(&env, &sender);

    client
        .mock_all_auths()
        .set_flow_limit(&token_id, &Some(i128::MAX - 50));

    let high_amount = i128::MAX - 100;
    let (source_chain, message_id, source_address, payload, messages) =
        create_interchain_transfer_message(&env, &client, &token_id, high_amount);
    approve_gateway_messages(&env, &gateway_client, signers, messages);
    client.execute(&source_chain, &message_id, &source_address, &payload);

    let small_amount = 100;
    let destination_chain = String::from_str(&env, "ethereum");
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &small_amount,
        &None,
        &gas_token,
    );
}
