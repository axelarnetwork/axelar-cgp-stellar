use soroban_sdk::{Address, Env, Vec};
use stellar_axelar_std::ensure;
use stellar_axelar_std::interfaces::CustomMigratableInterface;

use crate::error::ContractError;
use crate::{storage, AxelarOperators};

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
    type Error = ContractError;

    fn __migrate(env: &Env, migration_data: Self::MigrationData) -> Result<(), Self::Error> {
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
}
