use axelar_soroban_std::ttl::extend_instance_ttl;
use axelar_soroban_std::{ensure, interfaces, Ownable, Upgradable};
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};

use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;

#[contract]
#[derive(Ownable, Upgradable)]
pub struct AxelarOperators;

#[contractimpl]
impl AxelarOperators {
    pub fn __constructor(env: Env, owner: Address) {
        interfaces::set_owner(&env, &owner);
    }

    /// Return true if the account is an operator.
    pub fn is_operator(env: Env, account: Address) -> bool {
        let key = DataKey::Operators(account);

        env.storage().instance().has(&key)
    }

    /// Add an address as an operator.
    ///
    /// Only callable by the contract owner.
    pub fn add_operator(env: Env, account: Address) -> Result<(), ContractError> {
        Self::owner(&env).require_auth();

        let key = DataKey::Operators(account.clone());

        ensure!(
            !env.storage().instance().has(&key),
            ContractError::OperatorAlreadyAdded
        );

        env.storage().instance().set(&key, &true);

        extend_instance_ttl(&env);

        event::add_operator(&env, account);
        Ok(())
    }

    /// Remove an address as an operator.
    ///
    /// Only callable by the contract owner.
    pub fn remove_operator(env: Env, account: Address) -> Result<(), ContractError> {
        Self::owner(&env).require_auth();

        let key = DataKey::Operators(account.clone());

        ensure!(
            env.storage().instance().has(&key),
            ContractError::NotAnOperator
        );

        env.storage().instance().remove(&key);

        event::remove_operator(&env, account);
        Ok(())
    }

    /// Execute a function on a contract as an operator.
    pub fn execute(
        env: Env,
        operator: Address,
        contract: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> Result<Val, ContractError> {
        operator.require_auth();

        let key = DataKey::Operators(operator);

        ensure!(
            env.storage().instance().has(&key),
            ContractError::NotAnOperator
        );

        let res: Val = env.invoke_contract(&contract, &func, args);

        extend_instance_ttl(&env);

        Ok(res)
    }
}

impl AxelarOperators {
    // Modify this function to add migration logic
    const fn run_migration(_env: &Env, _migration_data: ()) {}
}
