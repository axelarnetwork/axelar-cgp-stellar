mod utils;

use axelar_gateway::types::Message as GatewayMessage;
use axelar_soroban_std::events;
use interchain_token_service::event::{
    InterchainTokenDeployedEvent, InterchainTransferReceivedEvent,
};
use interchain_token_service::types::{
    DeployInterchainToken, HubMessage, InterchainTransfer, Message, TokenManagerType,
};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, BytesN, String};
use soroban_token_sdk::metadata::TokenMetadata;
use utils::{approve_gateway_messages, register_chains, setup_env, setup_its_token, HUB_CHAIN};

use interchain_token::InterchainTokenClient;

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // ExecutableError::NotApproved
fn execute_fails_without_gateway_approval() {
    let (env, client, _, _, _) = setup_env();

    let source_chain = String::from_str(&env, "chain");
    let message_id = String::from_str(&env, "test");
    let source_address = String::from_str(&env, "source");
    let payload = Bytes::new(&env);

    client.execute(&source_chain, &message_id, &source_address, &payload);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")] // ContractError::InsufficientMessageLength
fn execute_fails_with_invalid_message() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let source_chain = client.its_hub_chain_name();
    let message_id = String::from_str(&env, "test");
    let source_address = Address::generate(&env).to_string();

    let invalid_payload = Bytes::from_array(&env, &[1u8; 16]);
    let payload_hash: BytesN<32> = env.crypto().keccak256(&invalid_payload).into();

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

    approve_gateway_messages(&env, &gateway_client, signers, messages);

    client.execute(
        &source_chain,
        &message_id,
        &source_address,
        &invalid_payload,
    );
}

#[test]
fn interchain_transfer_message_execute_succeeds() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);

    let sender = Address::generate(&env).to_xdr(&env);
    let recipient = Address::generate(&env).to_xdr(&env);
    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string();

    let amount = 1000;
    let deployer = Address::generate(&env);
    let token_id = setup_its_token(&env, &client, &deployer, amount);

    let msg = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(&env, HUB_CHAIN),
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: sender,
            destination_address: recipient,
            amount,
            data: None,
        }),
    };
    let payload = msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    let message_id = String::from_str(&env, "test");

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

    approve_gateway_messages(&env, &gateway_client, signers, messages);

    client.execute(&source_chain, &message_id, &source_address, &payload);

    goldie::assert!(events::fmt_last_emitted_event::<
        InterchainTransferReceivedEvent,
    >(&env));
}

#[test]
fn deploy_interchain_token_message_execute_succeeds() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);

    let sender = Address::generate(&env);
    let sender_bytes = sender.clone().to_xdr(&env);
    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string();

    let token_id = BytesN::from_array(&env, &[1u8; 32]);
    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "Test"),
        symbol: String::from_str(&env, "TEST"),
        decimal: 18,
    };

    let msg = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(&env, HUB_CHAIN),
        message: Message::DeployInterchainToken(DeployInterchainToken {
            token_id: token_id.clone(),
            name: token_metadata.name.clone(),
            symbol: token_metadata.symbol.clone(),
            decimals: token_metadata.decimal as u8,
            minter: Some(sender_bytes),
        }),
    };
    let payload = msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    let message_id = String::from_str(&env, "test");

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

    approve_gateway_messages(&env, &gateway_client, signers, messages);

    client.execute(&source_chain, &message_id, &source_address, &payload);

    goldie::assert!(events::fmt_last_emitted_event::<InterchainTokenDeployedEvent>(&env));

    let token = InterchainTokenClient::new(&env, &client.token_address(&token_id));

    assert!(token.is_minter(&sender));
    assert_eq!(token.name(), token_metadata.name);
    assert_eq!(token.symbol(), token_metadata.symbol);
    assert_eq!(token.decimals(), token_metadata.decimal);
    assert_eq!(
        client.token_manager_type(&token_id),
        TokenManagerType::NativeInterchainToken
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")] // ContractError::InvalidTokenMetadata
fn deploy_interchain_token_message_execute_fails_empty_token_name() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);

    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string();
    let token_id = BytesN::from_array(&env, &[1u8; 32]);

    let msg_empty_name = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(&env, HUB_CHAIN),
        message: Message::DeployInterchainToken(DeployInterchainToken {
            token_id,
            name: String::from_str(&env, ""),
            symbol: String::from_str(&env, "TEST"),
            decimals: 18,
            minter: None,
        }),
    };
    let payload_empty_name = msg_empty_name.abi_encode(&env).unwrap();
    let payload_hash_empty_name: BytesN<32> = env.crypto().keccak256(&payload_empty_name).into();

    let message_id_empty_name = String::from_str(&env, "no_name");

    let messages = vec![
        &env,
        GatewayMessage {
            source_chain: source_chain.clone(),
            message_id: message_id_empty_name.clone(),
            source_address: source_address.clone(),
            contract_address: client.address.clone(),
            payload_hash: payload_hash_empty_name,
        },
    ];

    approve_gateway_messages(&env, &gateway_client, signers, messages);

    client.execute(
        &source_chain,
        &message_id_empty_name,
        &source_address,
        &payload_empty_name,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")] // ContractError::InvalidTokenMetadata
fn deploy_interchain_token_message_execute_fails_empty_token_symbol() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);

    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string();
    let token_id = BytesN::from_array(&env, &[1u8; 32]);

    let msg_empty_symbol = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(&env, HUB_CHAIN),
        message: Message::DeployInterchainToken(DeployInterchainToken {
            token_id,
            name: String::from_str(&env, "test"),
            symbol: String::from_str(&env, ""),
            decimals: 18,
            minter: None,
        }),
    };
    let payload_empty_symbol = msg_empty_symbol.abi_encode(&env).unwrap();
    let payload_hash_empty_symbol: BytesN<32> =
        env.crypto().keccak256(&payload_empty_symbol).into();

    let message_id_empty_symbol = String::from_str(&env, "no_symbol");

    let messages = vec![
        &env,
        GatewayMessage {
            source_chain: source_chain.clone(),
            message_id: message_id_empty_symbol.clone(),
            source_address: source_address.clone(),
            contract_address: client.address.clone(),
            payload_hash: payload_hash_empty_symbol,
        },
    ];

    approve_gateway_messages(&env, &gateway_client, signers, messages);

    client.execute(
        &source_chain,
        &message_id_empty_symbol,
        &source_address,
        &payload_empty_symbol,
    );
}

#[test]
#[should_panic(expected = "Error(Value, InvalidInput)")]
fn deploy_interchain_token_message_execute_fails_invalid_minter_address() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);

    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string();
    let token_id = BytesN::from_array(&env, &[1u8; 32]);

    let invalid_minter = Bytes::from_array(&env, &[1u8; 32]);

    let msg_invalid_minter = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(&env, HUB_CHAIN),
        message: Message::DeployInterchainToken(DeployInterchainToken {
            token_id,
            name: String::from_str(&env, "test"),
            symbol: String::from_str(&env, "TEST"),
            decimals: 18,
            minter: Some(invalid_minter),
        }),
    };
    let payload_invalid_minter = msg_invalid_minter.abi_encode(&env).unwrap();
    let payload_hash_invalid_minter: BytesN<32> =
        env.crypto().keccak256(&payload_invalid_minter).into();

    let message_id_invalid_minter = String::from_str(&env, "invalid_minter");

    let messages = vec![
        &env,
        GatewayMessage {
            source_chain: source_chain.clone(),
            message_id: message_id_invalid_minter.clone(),
            source_address: source_address.clone(),
            contract_address: client.address.clone(),
            payload_hash: payload_hash_invalid_minter,
        },
    ];

    approve_gateway_messages(&env, &gateway_client, signers, messages);

    client.execute(
        &source_chain,
        &message_id_invalid_minter,
        &source_address,
        &payload_invalid_minter,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #18)")] // ContractError::TokenAlreadyDeployed
fn deploy_interchain_token_message_execute_fails_token_already_deployed() {
    let (env, client, gateway_client, _, signers) = setup_env();
    register_chains(&env, &client);

    let sender = Address::generate(&env).to_xdr(&env);
    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string();

    let token_id = BytesN::from_array(&env, &[1u8; 32]);
    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "Test"),
        symbol: String::from_str(&env, "TEST"),
        decimal: 18,
    };

    let msg = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(&env, HUB_CHAIN),
        message: Message::DeployInterchainToken(DeployInterchainToken {
            token_id,
            name: token_metadata.name.clone(),
            symbol: token_metadata.symbol.clone(),
            decimals: token_metadata.decimal as u8,
            minter: Some(sender),
        }),
    };
    let payload = msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    let first_message_id = String::from_str(&env, "first_deployment");
    let second_message_id = String::from_str(&env, "second_deployment");

    let messages = vec![
        &env,
        GatewayMessage {
            source_chain: source_chain.clone(),
            message_id: first_message_id.clone(),
            source_address: source_address.clone(),
            contract_address: client.address.clone(),
            payload_hash: payload_hash.clone(),
        },
        GatewayMessage {
            source_chain: source_chain.clone(),
            message_id: second_message_id.clone(),
            source_address: source_address.clone(),
            contract_address: client.address.clone(),
            payload_hash,
        },
    ];

    approve_gateway_messages(&env, &gateway_client, signers, messages);

    client.execute(&source_chain, &first_message_id, &source_address, &payload);

    client.execute(&source_chain, &second_message_id, &source_address, &payload);
}
