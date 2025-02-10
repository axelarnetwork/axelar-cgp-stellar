use soroban_sdk::{Address, Env, Vec};

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

pub type MigrationData = Vec<Address>;

pub fn migrate(env: &Env, migration_data: MigrationData) {
    for account in migration_data {
        if storage::is_operators(env, account.clone()) {
            super::storage::set_operator_status(env, account.clone());
            storage::remove_operators_status(env, account);
        }
    }
}
