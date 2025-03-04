use soroban_sdk::{
    contract, contracterror, contractimpl, token, Address, Bytes, BytesN, Env, String,
};
use stellar_axelar_gas_service::AxelarGasServiceClient;
use stellar_axelar_gateway::executable::{AxelarExecutableInterface, NotApprovedError};
use stellar_axelar_gateway::{impl_not_approved_error, AxelarGatewayMessagingClient};
use stellar_axelar_std::events::Event;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{ensure, InterchainTokenExecutable};
use stellar_interchain_token_service::executable::CustomInterchainTokenExecutable;
use stellar_interchain_token_service::InterchainTokenServiceClient;

use crate::event::{ExecutedEvent, TokenReceivedEvent, TokenSentEvent};
use crate::interface::ExampleInterface;
use crate::storage;

#[contract]
#[derive(InterchainTokenExecutable)]
pub struct Example;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ExampleError {
    NotApproved = 1,
    InvalidItsAddress = 2,
    InvalidAmount = 3,
}

impl_not_approved_error!(ExampleError);

#[contractimpl]
impl AxelarExecutableInterface for Example {
    type Error = ExampleError;

    fn gateway(env: &Env) -> Address {
        storage::gateway(env)
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

impl CustomInterchainTokenExecutable for Example {
    type Error = ExampleError;

    fn __interchain_token_service(env: &Env) -> Address {
        storage::interchain_token_service(env)
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
    ) -> Result<(), Self::Error> {
        ensure!(amount >= 0, ExampleError::InvalidAmount);

        let destination_address = Address::from_string_bytes(&payload);

        let token = token::TokenClient::new(env, &token_address);
        token.transfer(
            &env.current_contract_address(),
            &destination_address,
            &amount,
        );

        TokenReceivedEvent {
            source_chain,
            message_id,
            source_address,
            token_id,
            token_address,
            amount,
            payload,
        }
        .emit(env);

        Ok(())
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
        storage::set_gateway(env, &gateway);
        storage::set_gas_service(env, &gas_service);
        storage::set_interchain_token_service(env, &interchain_token_service);
    }
}

#[contractimpl]
impl ExampleInterface for Example {
    fn gas_service(env: &Env) -> Address {
        storage::gas_service(env)
    }

    fn send(
        env: &Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        message: Bytes,
        gas_token: Option<Token>,
    ) {
        let gateway = AxelarGatewayMessagingClient::new(env, &Self::gateway(env));
        let gas_service = AxelarGasServiceClient::new(env, &Self::gas_service(env));

        caller.require_auth();

        if let Some(gas_token) = gas_token {
            gas_service.pay_gas(
                &env.current_contract_address(),
                &destination_chain,
                &destination_address,
                &message,
                &caller,
                &gas_token,
                &Bytes::new(env),
            );
        }

        gateway.call_contract(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
        );
    }

    fn send_token(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        destination_app_contract: Bytes,
        amount: i128,
        recipient: Option<Bytes>,
        gas_token: Option<Token>,
    ) -> Result<(), ExampleError> {
        caller.require_auth();

        let client = InterchainTokenServiceClient::new(env, &Self::interchain_token_service(env));

        client.interchain_transfer(
            &caller,
            &token_id,
            &destination_chain,
            &destination_app_contract,
            &amount,
            &recipient,
            &gas_token,
        );

        TokenSentEvent {
            sender: caller,
            token_id,
            destination_chain,
            destination_app_contract,
            amount,
            recipient,
        }
        .emit(env);

        Ok(())
    }
}
