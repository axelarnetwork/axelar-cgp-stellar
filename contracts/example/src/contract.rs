use soroban_sdk::{contract, contracterror, contractimpl, Address, Bytes, BytesN, Env, String};
use stellar_axelar_gas_service::AxelarGasServiceClient;
use stellar_axelar_gateway::executable::{AxelarExecutableInterface, NotApprovedError};
use stellar_axelar_gateway::{impl_not_approved_error, AxelarGatewayMessagingClient};
use stellar_axelar_std::events::Event;
use stellar_axelar_std::types::Token;
use stellar_interchain_token_service::executable::InterchainTokenExecutableInterface;
use stellar_interchain_token_service::InterchainTokenServiceClient;

use crate::event::{ExecutedEvent, TokenReceivedEvent, TokenSentEvent};
use crate::storage_types::DataKey;

#[contract]
pub struct Example;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ExampleError {
    NotApproved = 1,
    InvalidItsAddress = 2,
}

impl_not_approved_error!(ExampleError);

#[contractimpl]
impl AxelarExecutableInterface for Example {
    type Error = ExampleError;

    fn gateway(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Gateway).unwrap()
    }

    fn execute(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) -> Result<(), ExampleError> {
        Self::validate_message(&env, &source_chain, &message_id, &source_address, &payload)?;

        ExecutedEvent {
            source_chain,
            message_id,
            source_address,
            payload,
        }
        .emit(&env);

        Ok(())
    }
}

#[contractimpl]
impl InterchainTokenExecutableInterface for Example {
    fn interchain_token_service(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::InterchainTokenService)
            .expect("ITS not found")
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

        TokenReceivedEvent {
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
impl Example {
    pub fn __constructor(
        env: &Env,
        gateway: Address,
        gas_service: Address,
        interchain_token_service: Address,
    ) {
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
        env.storage()
            .instance()
            .set(&DataKey::InterchainTokenService, &interchain_token_service);
    }

    pub fn gas_service(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::GasService).unwrap()
    }

    pub fn send(
        env: &Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        message: Bytes,
        gas_token: Token,
    ) {
        let gateway = AxelarGatewayMessagingClient::new(&env, &Self::gateway(&env));
        let gas_service = AxelarGasServiceClient::new(&env, &Self::gas_service(&env));

        caller.require_auth();

        gas_service.pay_gas(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
            &caller,
            &gas_token,
            &Bytes::new(&env),
        );

        gateway.call_contract(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
        );
    }

    pub fn send_token(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        destination_address: Bytes,
        amount: i128,
        message: Option<Bytes>,
        gas_token: Token,
    ) -> Result<(), ExampleError> {
        caller.require_auth();

        let interchain_token_service = env
            .storage()
            .instance()
            .get(&DataKey::InterchainTokenService)
            .ok_or(ExampleError::InvalidItsAddress)?;

        let interchain_token_service_client =
            InterchainTokenServiceClient::new(&env, &interchain_token_service);

        interchain_token_service_client.interchain_transfer(
            &caller,
            &token_id,
            &destination_chain,
            &destination_address,
            &amount,
            &message,
            &gas_token,
        );

        TokenSentEvent {
            sender: caller,
            token_id,
            destination_chain,
            destination_address,
            amount,
            message,
        }
        .emit(&env);

        Ok(())
    }
}
