mod utils;

use stellar_axelar_gateway::testutils::{generate_proof, get_approve_hash};
use stellar_axelar_gateway::types::Message as GatewayMessage;
use stellar_axelar_soroban_std::traits::BytesExt;
use stellar_axelar_soroban_std::{assert_invoke_auth_err, events};
use stellar_interchain_token_service::types::{HubMessage, InterchainTransfer, Message};
use soroban_sdk::token;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, BytesN, String};
use utils::{register_chains, setup_env, setup_its_token, HUB_CHAIN};

mod test {
    use stellar_axelar_soroban_std::{events::Event, impl_event_testutils};
    use core::fmt::Debug;
    use stellar_interchain_token_service::executable::InterchainTokenExecutableInterface;
    use soroban_sdk::{
        contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, IntoVal, String, Symbol,
        Topics, Val,
    };

    #[contract]
    pub struct ExecutableContract;

    #[contracttype]
    #[derive(Clone, Debug)]
    pub enum DataKey {
        InterchainTokenService,
        Message,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct ExecutedEvent {
        pub source_chain: String,
        pub message_id: String,
        pub source_address: Bytes,
        pub payload: Bytes,
        pub token_id: BytesN<32>,
        pub token_address: Address,
        pub amount: i128,
    }

    impl Event for ExecutedEvent {
        fn topics(&self, env: &Env) -> impl Topics + Debug {
            (
                Symbol::new(env, "executed"),
                self.source_chain.to_val(),
                self.message_id.to_val(),
                self.source_address.to_val(),
                self.token_id.to_val(),
                self.token_address.to_val(),
                self.amount,
            )
        }

        fn data(&self, _env: &Env) -> impl IntoVal<Env, Val> + Debug {
            (self.payload.to_val(),)
        }
    }

    impl_event_testutils!(
        ExecutedEvent,
        (Symbol, String, String, Bytes, BytesN<32>, Address, i128),
        (Bytes)
    );

    #[contractimpl]
    impl InterchainTokenExecutableInterface for ExecutableContract {
        fn interchain_token_service(env: &Env) -> Address {
            env.storage()
                .instance()
                .get(&DataKey::InterchainTokenService)
                .expect("its not found")
        }

        fn execute_with_interchain_token(
            env: &Env,
            source_chain: String,
            message_id: String,
            source_address: Bytes,
            payload: Bytes,
            token_id: BytesN<32>,
            token_address: Address,
            amount: i128,
        ) {
            Self::validate(env);

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
    register_chains(&env, &client);

    let executable_id = env.register(test::ExecutableContract, (client.address.clone(),));

    let sender = Address::generate(&env).to_xdr(&env);
    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string();

    let amount = 1000;
    let deployer = Address::generate(&env);
    let token_id = setup_its_token(&env, &client, &deployer, amount);
    let data = Bytes::from_hex(&env, "dead");

    let msg = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(&env, HUB_CHAIN),
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id: token_id.clone(),
            source_address: sender,
            destination_address: executable_id.clone().to_xdr(&env),
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

    let token = token::TokenClient::new(&env, &client.token_address(&token_id));
    assert_eq!(token.balance(&executable_id), amount);

    goldie::assert!(events::fmt_last_emitted_event::<test::ExecutedEvent>(&env));

    let executable_client = test::ExecutableContractClient::new(&env, &executable_id);
    assert_eq!(executable_client.message(), Some(data));
}

#[test]
fn executable_fails_if_not_executed_from_its() {
    let (env, client, _, _, _) = setup_env();

    let executable_id = env.register(test::ExecutableContract, (client.address.clone(),));
    let executable_client = test::ExecutableContractClient::new(&env, &executable_id);

    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_xdr(&env);
    let amount = 1000;
    let token_id = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_address = Address::generate(&env);
    let message_id = String::from_str(&env, "test");
    let payload = Bytes::from_hex(&env, "dead");

    assert_invoke_auth_err!(
        Address::generate(&env),
        executable_client.try_execute_with_interchain_token(
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
