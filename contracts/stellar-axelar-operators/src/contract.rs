use crate::error::ContractError;
use crate::event::{OperatorAddedEvent, OperatorRemovedEvent};
use crate::interface::AxelarOperatorsInterface;
use crate::storage_types::DataKey;
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};
use stellar_axelar_std::events::Event;
use stellar_axelar_std::interfaces::CustomMigratableInterface;
use stellar_axelar_std::ttl::extend_instance_ttl;
use stellar_axelar_std::{ensure, interfaces, Ownable, Upgradable};

#[contract]
#[derive(Ownable, Upgradable)]
pub struct AxelarOperators;

#[contractimpl]
impl AxelarOperators {
    pub fn __constructor(env: Env, owner: Address) {
        interfaces::set_owner(&env, &owner);
    }
}

#[contractimpl]
impl AxelarOperatorsInterface for AxelarOperators {
    fn is_operator(env: Env, account: Address) -> bool {
        let key = DataKey::Operators(account);

        env.storage().instance().has(&key)
    }

    fn add_operator(env: Env, account: Address) -> Result<(), ContractError> {
        Self::owner(&env).require_auth();

        let key = DataKey::Operators(account.clone());

        ensure!(
            !env.storage().instance().has(&key),
            ContractError::OperatorAlreadyAdded
        );

        env.storage().instance().set(&key, &true);

        extend_instance_ttl(&env);

        OperatorAddedEvent { operator: account }.emit(&env);

        Ok(())
    }

    fn remove_operator(env: Env, account: Address) -> Result<(), ContractError> {
        Self::owner(&env).require_auth();

        let key = DataKey::Operators(account.clone());

        ensure!(
            env.storage().instance().has(&key),
            ContractError::NotAnOperator
        );

        env.storage().instance().remove(&key);

        OperatorRemovedEvent { operator: account }.emit(&env);

        Ok(())
    }

    fn execute(
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

impl CustomMigratableInterface for AxelarOperators {
    type MigrationData = ();
    fn __migrate(_env: &Env, _migration_data: ()) {}
}
