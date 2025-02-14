use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env, String};
use stellar_axelar_std::events::Event;
use stellar_axelar_std::ttl::extend_instance_ttl;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{ensure, interfaces, only_operator, Operatable, Ownable, Upgradable};

use crate::error::ContractError;
use crate::event::{GasAddedEvent, GasCollectedEvent, GasPaidEvent, GasRefundedEvent};
use crate::interface::AxelarGasServiceInterface;

#[contract]
#[derive(Operatable, Ownable, Upgradable)]
pub struct AxelarGasService;

#[contractimpl]
impl AxelarGasService {
    /// Initialize the gas service contract with a gas_collector address.
    pub fn __constructor(env: Env, owner: Address, operator: Address) {
        interfaces::set_operator(&env, &operator);
        interfaces::set_owner(&env, &owner);
    }
}

#[contractimpl]
impl AxelarGasServiceInterface for AxelarGasService {
    fn pay_gas(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        spender: Address,
        token: Token,
        metadata: Bytes,
    ) -> Result<(), ContractError> {
        spender.require_auth();

        ensure!(token.amount > 0, ContractError::InvalidAmount);

        token::Client::new(&env, &token.address).transfer(
            &spender,
            &env.current_contract_address(),
            &token.amount,
        );

        GasPaidEvent {
            sender,
            destination_chain,
            destination_address,
            payload_hash: env.crypto().keccak256(&payload).into(),
            spender,
            token,
            metadata,
        }
        .emit(&env);

        Ok(())
    }

    fn add_gas(
        env: Env,
        sender: Address,
        message_id: String,
        spender: Address,
        token: Token,
    ) -> Result<(), ContractError> {
        spender.require_auth();

        ensure!(token.amount > 0, ContractError::InvalidAmount);

        token::Client::new(&env, &token.address).transfer(
            &spender,
            &env.current_contract_address(),
            &token.amount,
        );

        GasAddedEvent {
            sender,
            message_id,
            spender,
            token,
        }
        .emit(&env);

        Ok(())
    }

    #[only_operator]
    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError> {
        ensure!(token.amount > 0, ContractError::InvalidAmount);

        let token_client = token::Client::new(&env, &token.address);

        let contract_token_balance = token_client.balance(&env.current_contract_address());

        ensure!(
            contract_token_balance >= token.amount,
            ContractError::InsufficientBalance
        );
        token_client.transfer(&env.current_contract_address(), &receiver, &token.amount);

        GasCollectedEvent { receiver, token }.emit(&env);

        extend_instance_ttl(&env);

        Ok(())
    }

    #[only_operator]
    fn refund(env: Env, message_id: String, receiver: Address, token: Token) {
        token::Client::new(&env, &token.address).transfer(
            &env.current_contract_address(),
            &receiver,
            &token.amount,
        );

        GasRefundedEvent {
            message_id,
            receiver,
            token,
        }
        .emit(&env);
    }
}
