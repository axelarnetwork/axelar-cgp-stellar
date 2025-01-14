use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::executable::{AxelarExecutableInterface, NotApprovedError};
use axelar_gateway::{impl_not_approved_error, AxelarGatewayMessagingClient};
use axelar_soroban_std::types::Token;
use soroban_sdk::{contract, contracterror, contractimpl, Address, Bytes, Env, String};

use crate::event;
use crate::storage_types::DataKey;

#[contract]
pub struct Example;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ExampleError {
    NotApproved = 1,
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

        event::executed(&env, source_chain, message_id, source_address, payload);
        Ok(())
    }
}

#[contractimpl]
impl Example {
    pub fn __constructor(env: Env, gateway: Address, gas_service: Address) {
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
    }

    pub fn gas_service(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::GasService).unwrap()
    }

    pub fn send(
        env: Env,
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
}
