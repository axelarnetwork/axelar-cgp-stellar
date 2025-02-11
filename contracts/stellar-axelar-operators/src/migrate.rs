use soroban_sdk::{Address, Env, Vec};
use stellar_axelar_std::ensure;

use crate::{error::ContractError, storage};

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

pub type MigrationData = Vec<Address>;

pub fn migrate(env: &Env, migration_data: MigrationData) -> Result<(), ContractError> {
    for account in migration_data {
        ensure!(
            legacy_storage::is_operators(env, account.clone()),
            ContractError::NotAnOperator
        );

        storage::set_operator_status(env, account.clone());
        legacy_storage::remove_operators_status(env, account);
    }

    Ok(())
}
