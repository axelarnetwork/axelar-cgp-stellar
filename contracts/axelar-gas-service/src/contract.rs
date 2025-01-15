use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env, String};

use crate::error::ContractError;
use crate::event;
use crate::interface::AxelarGasServiceInterface;
use crate::storage_types::DataKey;
use stellar_axelar_std::ttl::extend_instance_ttl;
use stellar_axelar_std::{ensure, interfaces, types::Token};
use stellar_axelar_std::{Ownable, Upgradable};

#[contract]
#[derive(Ownable, Upgradable)]
pub struct AxelarGasService;

#[contractimpl]
impl AxelarGasService {
    /// Initialize the gas service contract with a gas_collector address.
    pub fn __constructor(env: Env, owner: Address, gas_collector: Address) {
        interfaces::set_owner(&env, &owner);
        env.storage()
            .instance()
            .set(&DataKey::GasCollector, &gas_collector);
    }
}

impl AxelarGasService {
    // Modify this function to add migration logic
    const fn run_migration(_env: &Env, _migration_data: ()) {}
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

        event::gas_paid(
            &env,
            sender,
            destination_chain,
            destination_address,
            payload,
            spender,
            token,
            metadata,
        );

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

        event::gas_added(&env, sender, message_id, spender, token);

        Ok(())
    }

    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError> {
        let gas_collector = Self::gas_collector(&env);
        gas_collector.require_auth();

        ensure!(token.amount > 0, ContractError::InvalidAmount);

        let token_client = token::Client::new(&env, &token.address);

        let contract_token_balance = token_client.balance(&env.current_contract_address());

        ensure!(
            contract_token_balance >= token.amount,
            ContractError::InsufficientBalance
        );
        token_client.transfer(&env.current_contract_address(), &receiver, &token.amount);

        event::fee_collected(&env, gas_collector, token);

        extend_instance_ttl(&env);

        Ok(())
    }

    fn refund(env: Env, message_id: String, receiver: Address, token: Token) {
        Self::gas_collector(&env).require_auth();

        token::Client::new(&env, &token.address).transfer(
            &env.current_contract_address(),
            &receiver,
            &token.amount,
        );

        event::refunded(&env, message_id, receiver, token);
    }

    fn gas_collector(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::GasCollector)
            .expect("gas collector not found")
    }
}
