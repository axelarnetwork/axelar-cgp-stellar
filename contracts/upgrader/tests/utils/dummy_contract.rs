//! Dummy contract to test the [crate::Upgrader]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env};
use stellar_axelar_std::interfaces;
use stellar_axelar_std::interfaces::{OwnableInterface, UpgradableInterface};

#[contract]
pub struct DummyContract;

#[contractimpl]
impl UpgradableInterface for DummyContract {
    fn version(env: &Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(env, "0.1.0")
    }

    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        Self::owner(env).require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}

#[contractimpl]
impl OwnableInterface for DummyContract {
    fn owner(env: &Env) -> Address {
        interfaces::owner(env)
    }

    fn transfer_ownership(env: &Env, new_owner: Address) {
        interfaces::transfer_ownership::<Self>(env, new_owner);
    }
}

#[contractimpl]
impl DummyContract {
    pub fn __constructor(env: Env, owner: Address) {
        interfaces::set_owner(&env, &owner);
    }
}

#[contracttype]
pub enum DataKey {
    Data,
}

#[contracterror]
pub enum ContractError {
    SomeFailure = 1,
}
