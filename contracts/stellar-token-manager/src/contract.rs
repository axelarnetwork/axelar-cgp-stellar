use crate::error::ContractError;
use crate::interface::TokenManagerInterface;
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};
use stellar_axelar_std::interfaces::CustomMigratableInterface;
use stellar_axelar_std::ttl::extend_instance_ttl;
use stellar_axelar_std::{interfaces, Ownable, Upgradable};

#[contract]
#[derive(Ownable, Upgradable)]
pub struct TokenManager;

#[contractimpl]
impl TokenManager {
    pub fn __constructor(env: &Env, owner: Address) {
        interfaces::set_owner(env, &owner);
    }
}

#[contractimpl]
impl TokenManagerInterface for TokenManager {
    fn execute(
        env: &Env,
        contract: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> Result<Val, ContractError> {
        Self::owner(env).require_auth();

        let res: Val = env.invoke_contract(&contract, &func, args);

        extend_instance_ttl(env);

        Ok(res)
    }
}

impl CustomMigratableInterface for TokenManager {
    type MigrationData = ();

    fn __migrate(_env: &Env, _migration_data: Self::MigrationData) {}
}
