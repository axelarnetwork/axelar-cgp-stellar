use soroban_sdk::{
    assert_with_error, contract, contracterror, contractimpl, token, Address, Bytes, BytesN, Env,
    String,
};
use stellar_axelar_gas_service::AxelarGasServiceClient;
use stellar_axelar_gateway::executable::{AxelarExecutableInterface, NotApprovedError};
use stellar_axelar_gateway::{impl_not_approved_error, AxelarGatewayMessagingClient};
use stellar_axelar_std::events::Event;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::InterchainTokenExecutable;
use stellar_interchain_token_service::executable::CustomInterchainTokenExecutable;
use stellar_interchain_token_service::InterchainTokenServiceClient;

use crate::event::{ExecutedEvent, TokenReceivedEvent, TokenSentEvent};
use crate::storage_types::DataKey;

#[contract]
#[derive(InterchainTokenExecutable)]
pub struct Example;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
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
        env.storage()
            .instance()
            .get(&DataKey::Gateway)
            .expect("gateway not found")
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
impl CustomInterchainTokenExecutable for Example {
    type Error = ExampleError;

    fn __interchain_token_service(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::InterchainTokenService)
            .expect("ITS not found")
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
        Self::validate_amount(env, amount)?;

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
        env.storage()
            .instance()
            .get(&DataKey::GasService)
            .expect("gas service not found")
    }

    pub fn send(
        env: &Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        message: Bytes,
        gas_token: Token,
    ) {
        let gateway = AxelarGatewayMessagingClient::new(env, &Self::gateway(env));
        let gas_service = AxelarGasServiceClient::new(env, &Self::gas_service(env));

        caller.require_auth();

        gas_service.pay_gas(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
            &caller,
            &gas_token,
            &Bytes::new(env),
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

        let client = InterchainTokenServiceClient::new(env, &Self::interchain_token_service(env));

        client.interchain_transfer(
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
        .emit(env);

        Ok(())
    }

    pub fn validate_amount(env: &Env, amount: i128) -> Result<(), ExampleError> {
        assert_with_error!(env, amount >= 0, ExampleError::InvalidAmount);

        Ok(())
    }
}
