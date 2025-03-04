// this is only needed in this crate itself, any crate that imports this one doesn't have to do this manual import resolution
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};
use stellar_axelar_std_derive::{Ownable, Upgradable};

use crate as stellar_axelar_std;
use crate::interfaces::testdata::contract_trivial_migration::DataKey;
use crate::interfaces::{operatable, ownable, CustomMigratableInterface, MigratableInterface};

#[derive(Upgradable, Ownable)]
#[migratable]
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
}

impl CustomMigratableInterface for ContractNonTrivial {
    type MigrationData = MigrationData;
    type Error = ContractError;

    fn __migrate(env: &Env, migration_data: MigrationData) -> Result<(), Self::Error> {
        env.storage()
            .instance()
            .set(&DataKey::Data, &migration_data.data1);

        Ok(())
    }
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
}
