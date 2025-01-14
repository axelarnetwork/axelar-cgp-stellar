// this is only needed in this crate itself, any crate that imports this one doesn't have to do this manual import resolution
use axelar_soroban_std_derive::{Ownable, Upgradable};
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

use crate as axelar_soroban_std;
use crate::interfaces::testdata::contract_trivial_migration::DataKey;
use crate::interfaces::{operatable, ownable, MigratableInterface};

#[derive(Upgradable, Ownable)]
#[migratable(with_type = MigrationData)]
#[contract]
pub struct ContractNonTrivial;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationData {
    pub data1: String,
    pub data2: bool,
    pub data3: u32,
}

#[contractimpl]
impl ContractNonTrivial {
    pub fn __constructor(_env: Env, owner: Option<Address>, operator: Option<Address>) {
        if let Some(owner) = owner {
            ownable::set_owner(&_env, &owner);
        }

        if let Some(operator) = operator {
            operatable::set_operator(&_env, &operator);
        }
    }

    pub fn migration_data(env: &Env) -> Option<String> {
        env.storage().instance().get(&DataKey::Data)
    }

    fn run_migration(env: &Env, migration_data: MigrationData) {
        env.storage()
            .instance()
            .set(&DataKey::Data, &migration_data.data1);
    }
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
}
