use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};
use stellar_axelar_std::events::Event;
use stellar_axelar_std::ttl::extend_instance_ttl;
use stellar_axelar_std::{ensure, interfaces, only_owner, Ownable, Upgradable};

use crate::error::ContractError;
use crate::event::{OperatorAddedEvent, OperatorRemovedEvent};
use crate::interface::AxelarOperatorsInterface;
use crate::{migrate, storage};

#[contract]
#[derive(Ownable, Upgradable)]
#[migratable(with_type = migrate::MigrationData)]
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
        storage::is_operator(&env, account)
    }

    #[only_owner]
    fn add_operator(env: Env, account: Address) -> Result<(), ContractError> {
        ensure!(
            !storage::is_operator(&env, account.clone()),
            ContractError::OperatorAlreadyAdded
        );

        storage::set_operator_status(&env, account.clone());

        extend_instance_ttl(&env);

        OperatorAddedEvent { operator: account }.emit(&env);

        Ok(())
    }

    #[only_owner]
    fn remove_operator(env: Env, account: Address) -> Result<(), ContractError> {
        ensure!(
            storage::is_operator(&env, account.clone()),
            ContractError::NotAnOperator
        );

        storage::remove_operator_status(&env, account.clone());

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

        ensure!(
            storage::is_operator(&env, operator),
            ContractError::NotAnOperator
        );

        let res: Val = env.invoke_contract(&contract, &func, args);

        extend_instance_ttl(&env);

        Ok(res)
    }
}
