use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, vec, Address, Bytes, BytesN, String};
use stellar_axelar_gateway::testutils::{generate_proof, get_approve_hash};
use stellar_axelar_gateway::types::Message as GatewayMessage;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::traits::BytesExt;
use stellar_axelar_std::{assert_auth_err, events};

use super::utils::setup_env;
use crate::testutils::setup_its_token;
use crate::types::{HubMessage, InterchainTransfer, Message};

mod test {
    use core::fmt::Debug;

    use soroban_sdk::{
        contract, contracterror, contractimpl, contracttype, Address, Bytes, BytesN, Env, String,
    };
    use stellar_axelar_std::events::Event;
    use stellar_axelar_std::{ensure, InterchainTokenExecutable, IntoEvent};

    use crate as stellar_interchain_token_service;
    use crate::executable::CustomInterchainTokenExecutable;

    #[contract]
    #[derive(InterchainTokenExecutable)]
    pub struct ExecutableContract;

    #[contracttype]
    #[derive(Clone, Debug)]
    pub enum DataKey {
        InterchainTokenService,
        Message,
    }

    #[derive(Debug, PartialEq, Eq, IntoEvent)]
    pub struct ExecutedEvent {
        pub source_chain: String,
        pub message_id: String,
        pub source_address: Bytes,
        pub token_id: BytesN<32>,
        pub token_address: Address,
        pub amount: i128,
        #[data]
        pub payload: Bytes,
    }

    #[contracterror]
    pub enum ContractError {
        PayloadLenOne = 1,
    }

    impl CustomInterchainTokenExecutable for ExecutableContract {
        type Error = ContractError;

        fn __interchain_token_service(env: &Env) -> Address {
            env.storage()
                .instance()
                .get(&DataKey::InterchainTokenService)
                .expect("its not found")
        }

        fn __authorized_execute_with_token(
            env: &Env,
            source_chain: String,
            message_id: String,
            source_address: Bytes,
            payload: Bytes,
            token_id: BytesN<32>,
            token_address: Address,
            amount: i128,
        ) -> Result<(), ContractError> {
            ensure!(payload.len() != 1, ContractError::PayloadLenOne);

            env.storage().persistent().set(&DataKey::Message, &payload);

            ExecutedEvent {
                source_chain,
                message_id,
                source_address,
                payload,
                token_id,
                token_address,
                amount,
            }
            .emit(env);

            Ok(())
        }
    }

    #[contractimpl]
    impl ExecutableContract {
        pub fn __constructor(env: &Env, interchain_token_service: Address) {
            env.storage()
                .instance()
                .set(&DataKey::InterchainTokenService, &interchain_token_service);
        }

        pub fn message(env: &Env) -> Option<Bytes> {
            env.storage()
                .persistent()
                .get::<_, Bytes>(&DataKey::Message)
        }
    }
}

#[test]
fn interchain_transfer_execute_succeeds() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let executable_id = env.register(test::ExecutableContract, (client.address.clone(),));

    let sender = Address::generate(&env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();

    let amount = 1000;
    let deployer = Address::generate(&env);
    let (token_id, _) = setup_its_token(&env, &client, &deployer, amount);
    let data = Bytes::from_hex(&env, "dead");
    let destination_address = executable_id.to_string_bytes();
    let original_source_chain = String::from_str(&env, "ethereum");
    client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let msg = HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id: token_id.clone(),
            source_address: sender,
            destination_address,
            amount,
            data: Some(data.clone()),
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
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    gateway_client.approve_messages(&messages, &proof);

    client.execute(&source_chain, &message_id, &source_address, &payload);
    goldie::assert!(events::fmt_last_emitted_event::<test::ExecutedEvent>(&env));

    let token = token::TokenClient::new(&env, &client.registered_token_address(&token_id));
    assert_eq!(token.balance(&executable_id), amount);

    let executable_client = test::ExecutableContractClient::new(&env, &executable_id);
    assert_eq!(executable_client.message(), Some(data));
}

#[test]
fn executable_fails_if_not_executed_from_its() {
    let (env, client, _, _, _) = setup_env();

    let executable_id = env.register(test::ExecutableContract, (client.address.clone(),));
    let executable_client = test::ExecutableContractClient::new(&env, &executable_id);

    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string_bytes();
    let amount = 1000;
    let token_id = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_address = Address::generate(&env);
    let message_id = String::from_str(&env, "test");
    let payload = Bytes::from_hex(&env, "dead");

    assert_auth_err!(
        Address::generate(&env),
        executable_client.execute_with_interchain_token(
            &source_chain,
            &message_id,
            &source_address,
            &payload,
            &token_id,
            &token_address,
            &amount,
        )
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // ContractError::PayloadLenOne
fn interchain_transfer_execute_fails_if_payload_is_len_one() {
    let (env, client, gateway_client, _, signers) = setup_env();

    let executable_id = env.register(test::ExecutableContract, (client.address.clone(),));

    let sender = Address::generate(&env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();

    let amount = 1000;
    let deployer = Address::generate(&env);
    let (token_id, _) = setup_its_token(&env, &client, &deployer, amount);
    let data_with_len_1 = Bytes::from_slice(&env, &[1]);
    let destination_address = executable_id.to_string_bytes();
    let original_source_chain = String::from_str(&env, "ethereum");
    client
        .mock_all_auths()
        .set_trusted_chain(&original_source_chain);

    let msg = HubMessage::ReceiveFromHub {
        source_chain: original_source_chain,
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: sender,
            destination_address,
            amount,
            data: Some(data_with_len_1),
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
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    gateway_client.approve_messages(&messages, &proof);

    client.execute(&source_chain, &message_id, &source_address, &payload);
}
