use soroban_sdk::{panic_with_error, Address, Env, Vec};
use stellar_axelar_std::interfaces::CustomMigratableInterface;

use crate::error::ContractError;
use crate::{storage, AxelarOperators};

pub type MigrationData = <AxelarOperators as CustomMigratableInterface>::MigrationData;

mod legacy_storage {
    use soroban_sdk::Address;
    use stellar_axelar_std::contractstorage;

    #[contractstorage]
    #[derive(Clone, Debug)]
    enum LegacyDataKey {
        #[instance]
        #[status]
        Operators { account: Address },
    }
}

impl CustomMigratableInterface for AxelarOperators {
    type MigrationData = Vec<Address>;

    fn __migrate(env: &Env, migration_data: Self::MigrationData) {
        for account in migration_data {
            if !legacy_storage::is_operators(env, account.clone()) {
                panic_with_error!(env, ContractError::NotAnOperator);
            }

            storage::set_operator_status(env, account.clone());
            legacy_storage::remove_operators_status(env, account);
        }
    }
}
