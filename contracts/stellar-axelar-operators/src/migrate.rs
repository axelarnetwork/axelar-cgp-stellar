use crate::AxelarOperators;
use soroban_sdk::{Address, Env, Vec};
use stellar_axelar_std::interfaces::CustomMigratableInterface;

pub type MigrationData = <AxelarOperators as CustomMigratableInterface>::MigrationData;

mod storage {
    use soroban_sdk::Address;
    use stellar_axelar_std::contractstorage;

    #[contractstorage]
    #[derive(Clone, Debug)]
    pub enum MigrationDataKey {
        #[instance]
        #[status]
        Operators { account: Address },
    }
}

impl CustomMigratableInterface for AxelarOperators {
    type MigrationData = Vec<Address>;

    fn __migrate(_env: &Env, _migration_data: Self::MigrationData) {
        for account in _migration_data {
            if storage::is_operators(_env, account.clone()) {
                super::storage::set_operator_status(_env, account.clone());
                storage::remove_operators_status(_env, account);
            }
        }
    }
}
