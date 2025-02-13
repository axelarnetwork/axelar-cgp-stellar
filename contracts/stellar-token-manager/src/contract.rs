use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};
use stellar_axelar_std::ttl::extend_instance_ttl;
use stellar_axelar_std::{interfaces, only_owner, Ownable, Upgradable};

use crate::error::ContractError;
use crate::interface::TokenManagerInterface;

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
    #[only_owner]
    fn execute(
        env: &Env,
        contract: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> Result<Val, ContractError> {
        let res: Val = env.invoke_contract(&contract, &func, args);

        extend_instance_ttl(env);

        Ok(res)
    }
}
