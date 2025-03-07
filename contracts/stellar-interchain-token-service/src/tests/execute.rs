use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gateway::testutils::approve_gateway_messages;
use stellar_axelar_gateway::types::Message as GatewayMessage;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::{assert_contract_err, events};
use stellar_interchain_token::InterchainTokenClient;

use super::utils::{setup_env, TokenMetadataExt};
use crate::error::ContractError;
use crate::event::{
    InterchainTokenDeployedEvent, InterchainTransferReceivedEvent, TokenManagerDeployedEvent,
};
use crate::testutils::setup_its_token;
use crate::types::{
    DeployInterchainToken, HubMessage, InterchainTransfer, Message, TokenManagerType,
};

#[test]
fn interchain_transfer_message_execute_succeeds() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let sender = Address::generate(&env).to_string_bytes();
    let recipient = Address::generate(&env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();
    let original_source_chain = String::from_str(&env, "ethereum");

    let amount = 1000;
    let deployer = Address::generate(&env);
    let (token_id, _) = setup_its_token(&env, &client, &deployer, amount);
    client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let msg = HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: sender,
            destination_address: recipient,
            amount,
            data: None,
        }),
    };
    let message_id = String::from_str(&env, "test");
    let payload = msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

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
fn interchain_transfer_message_canonical_token_execute_succeeds() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let sender = Address::generate(&env).to_string_bytes();
    let recipient = Address::generate(&env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();
    let original_source_chain = String::from_str(&env, "ethereum");

    let amount = 1000;
    let deployer = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(deployer).address();
    let token_id = client
        .mock_all_auths()
        .register_canonical_token(&token_address);
    let token_manager = client.deployed_token_manager(&token_id);

    StellarAssetClient::new(&env, &token_address)
        .mock_all_auths()
        .mint(&token_manager, &amount);

    client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let msg = HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: sender,
            destination_address: recipient.clone(),
            amount,
            data: None,
        }),
    };
    let message_id = String::from_str(&env, "test");
    let payload = msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

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

    assert_eq!(
        TokenClient::new(&env, &token_address).balance(&token_manager),
        0
    );
    assert_eq!(
        TokenClient::new(&env, &token_address).balance(&Address::from_string_bytes(&recipient)),
        amount
    );
}

#[test]
fn deploy_interchain_token_message_execute_succeeds() {
    let (env, client, gateway_client, _, signers) = setup_env();
    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let sender = Address::generate(&env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();

    let token_id = BytesN::from_array(&env, &[1u8; 32]);
    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "Test"),
        symbol: String::from_str(&env, "TEST"),
        decimal: 18,
    };
    let original_source_chain = String::from_str(&env, "ethereum");
    client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let msg = HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
        message: Message::DeployInterchainToken(DeployInterchainToken {
            token_id: token_id.clone(),
            name: token_metadata.name.clone(),
            symbol: token_metadata.symbol.clone(),
            decimals: token_metadata.decimal as u8,
            minter: Some(sender.clone()),
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

    let interchain_token_deployed_event =
        events::fmt_emitted_event_at_idx::<InterchainTokenDeployedEvent>(&env, -3);
    let token_manager_deployed_event =
        events::fmt_emitted_event_at_idx::<TokenManagerDeployedEvent>(&env, -2);

    let token = InterchainTokenClient::new(&env, &client.registered_token_address(&token_id));

    assert!(token.is_minter(&Address::from_string_bytes(&sender)));
    assert_eq!(token.name(), token_metadata.name);
    assert_eq!(token.symbol(), token_metadata.symbol);
    assert_eq!(token.decimals(), token_metadata.decimal);
    assert_eq!(
        client.token_manager_type(&token_id),
        TokenManagerType::NativeInterchainToken
    );

    goldie::assert!([
        interchain_token_deployed_event,
        token_manager_deployed_event
    ]
    .join("\n\n"));
}

#[test]
fn execute_fails_when_paused() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let source_chain = String::from_str(&env, "chain");
    let message_id = String::from_str(&env, "test");
    let source_address = String::from_str(&env, "source");
    let payload = Bytes::new(&env);
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

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

    client.mock_all_auths().pause();

    assert_contract_err!(
        client.try_execute(&source_chain, &message_id, &source_address, &payload,),
        ContractError::ContractPaused
    );
}

#[test]
fn execute_fails_without_gateway_approval() {
    let (env, client, _, _, _) = setup_env();

    let source_chain = String::from_str(&env, "chain");
    let message_id = String::from_str(&env, "test");
    let source_address = String::from_str(&env, "source");
    let payload = Bytes::new(&env);

    assert_contract_err!(
        client.try_execute(&source_chain, &message_id, &source_address, &payload),
        ContractError::NotApproved
    );
}

#[test]
fn execute_fails_with_invalid_payload_length() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let message_id = String::from_str(&env, "test");
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();

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

    assert_contract_err!(
        client.try_execute(
            &source_chain,
            &message_id,
            &source_address,
            &invalid_payload,
        ),
        ContractError::InsufficientMessageLength
    );
}

#[test]
fn execute_fails_with_invalid_message_type() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let sender = Address::generate(&env).to_string_bytes();
    let recipient = Address::generate(&env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();
    let original_source_chain = String::from_str(&env, "ethereum");

    let amount = 1000;
    let deployer = Address::generate(&env);
    let (token_id, _) = setup_its_token(&env, &client, &deployer, amount);
    client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let invalid_msg = HubMessage::SendToHub {
        destination_chain: original_source_chain,
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: sender,
            destination_address: recipient,
            amount,
            data: None,
        }),
    };
    let message_id = String::from_str(&env, "test");
    let payload = invalid_msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

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

    assert_contract_err!(
        client.try_execute(&source_chain, &message_id, &source_address, &payload),
        ContractError::InvalidMessageType
    );
}

#[test]
fn execute_fails_with_invalid_source_chain() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let message_id = String::from_str(&env, "test");
    let source_chain = String::from_str(&env, "invalid");
    let source_address = client.its_hub_address();
    let payload = Bytes::new(&env);
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();
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

    assert_contract_err!(
        client.try_execute(&source_chain, &message_id, &source_address, &payload,),
        ContractError::NotHubChain
    );
}

#[test]
fn execute_fails_with_invalid_source_address() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let message_id = String::from_str(&env, "test");
    let source_chain = client.its_hub_chain_name();
    let source_address = String::from_str(&env, "invalid");
    let payload = Bytes::new(&env);
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();
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

    assert_contract_err!(
        client.try_execute(&source_chain, &message_id, &source_address, &payload,),
        ContractError::NotHubAddress
    );
}

#[test]
fn execute_fails_with_untrusted_chain() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let sender = Address::generate(&env).to_string_bytes();
    let recipient = Address::generate(&env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();
    let untrusted_chain = String::from_str(&env, "untrusted");
    let token_id = BytesN::from_array(&env, &[1u8; 32]);

    let msg = HubMessage::ReceiveFromHub {
        source_chain: untrusted_chain,
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: sender,
            destination_address: recipient,
            amount: 1,
            data: None,
        }),
    };
    let message_id = String::from_str(&env, "test");
    let payload = msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

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

    assert_contract_err!(
        client.try_execute(&source_chain, &message_id, &source_address, &payload),
        ContractError::UntrustedChain
    );
}

#[test]
fn execute_fails_with_invalid_amount() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let sender = Address::generate(&env).to_string_bytes();
    let recipient = Address::generate(&env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();
    let original_source_chain = String::from_str(&env, "ethereum");

    let invalid_amount = 0;
    let deployer = Address::generate(&env);
    let (token_id, _) = setup_its_token(&env, &client, &deployer, invalid_amount);
    client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let msg = HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: sender,
            destination_address: recipient,
            amount: invalid_amount,
            data: None,
        }),
    };
    let message_id = String::from_str(&env, "test");
    let payload = msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

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

    assert_contract_err!(
        client.try_execute(&source_chain, &message_id, &source_address, &payload),
        ContractError::InvalidAmount
    );
}

#[test]
fn deploy_interchain_token_message_execute_fails_invalid_token_metadata() {
    let env = Env::default();

    let cases = [
        (
            TokenMetadata::new(&env, "", "symbol", 6),
            ContractError::InvalidTokenName,
        ),
        (
            TokenMetadata::new(&env, "A".repeat(33).as_str(), "symbol", 6),
            ContractError::InvalidTokenName,
        ),
        (
            TokenMetadata::new(&env, "name", "", 6),
            ContractError::InvalidTokenSymbol,
        ),
        (
            TokenMetadata::new(&env, "name", "A".repeat(33).as_str(), 6),
            ContractError::InvalidTokenSymbol,
        ),
    ];

    for (
        i,
        (
            TokenMetadata {
                name,
                symbol,
                decimal,
            },
            expected_error,
        ),
    ) in cases.into_iter().enumerate()
    {
        let (env, client, gateway_client, _, signers) = setup_env();

        let source_chain = client.its_hub_chain_name();
        let source_address = client.its_hub_address();
        let original_source_chain = String::from_str(&env, "ethereum");
        let message_id = String::from_str(&env, "message_id");

        client
            .mock_all_auths()
            .set_trusted_chain(&original_source_chain);

        let token_id = BytesN::from_array(&env, &[i as u8; 32]);
        let msg = HubMessage::ReceiveFromHub {
            source_chain: original_source_chain.clone(),
            message: Message::DeployInterchainToken(DeployInterchainToken {
                token_id,
                name,
                symbol,
                decimals: decimal as u8,
                minter: None,
            }),
        };
        let payload = msg.abi_encode(&env).unwrap();
        let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

        let messages = vec![
            &env,
            GatewayMessage {
                source_chain: source_chain.clone(),
                message_id: message_id.clone(),
                source_address: source_address.clone(),
                contract_address: client.address.clone(),
                payload_hash: payload_hash.clone(),
            },
        ];

        approve_gateway_messages(&env, &gateway_client, signers, messages);

        assert_contract_err!(
            client.try_execute(&source_chain, &message_id, &source_address, &payload),
            expected_error
        );
    }
}

#[test]
#[should_panic(expected = "Error(Value, InvalidInput)")]
fn deploy_interchain_token_message_execute_fails_invalid_minter_address() {
    let (env, client, gateway_client, _, signers) = setup_env();
    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();
    let token_id = BytesN::from_array(&env, &[1u8; 32]);
    let invalid_minter = Bytes::from_array(&env, &[1u8; 32]);
    let original_source_chain = String::from_str(&env, "ethereum");
    client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let msg_invalid_minter = HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
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
fn deploy_interchain_token_message_execute_fails_token_already_deployed() {
    let (env, client, gateway_client, _, signers) = setup_env();
    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let sender = Address::generate(&env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();

    let token_id = BytesN::from_array(&env, &[1u8; 32]);
    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "Test"),
        symbol: String::from_str(&env, "TEST"),
        decimal: 18,
    };
    let original_source_chain = String::from_str(&env, "ethereum");
    client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let msg = HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
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

    assert_contract_err!(
        client.try_execute(&source_chain, &second_message_id, &source_address, &payload),
        ContractError::TokenAlreadyRegistered
    );
}
