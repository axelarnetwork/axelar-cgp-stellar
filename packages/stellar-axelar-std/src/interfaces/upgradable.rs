use core::fmt::Debug;

use soroban_sdk::{contractclient, BytesN, Env, FromVal, String, Val};

use crate as stellar_axelar_std;
use crate::events::Event;
use crate::interfaces::{storage, OwnableInterface};
use crate::{ensure, IntoEvent};

#[contractclient(name = "UpgradableClient")]
pub trait UpgradableInterface: OwnableInterface {
    /// Returns the current version of the contract.
    fn version(env: &Env) -> String;

    /// Upgrades the contract to a new WASM hash.
    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>);
}

pub trait MigratableInterface: UpgradableInterface + CustomMigratableInterface {
    /// Error type returned if the migration fails.
    type Error: Into<soroban_sdk::Error>;

    /// Migrates contract state after upgrading the contract code.
    fn migrate(
        env: &Env,
        migration_data: Self::MigrationData,
    ) -> Result<(), <Self as MigratableInterface>::Error>;
}

/// This trait is used to implement custom migration logic for a contract.
/// It is automatically implemented for the contract if the `#[migratable]` attribute is applied to the contract struct.
pub trait CustomMigratableInterface: UpgradableInterface {
    /// Data needed during the migration. Each contract can define its own data type.
    /// Choose `()` if none is necessary
    type MigrationData: FromVal<Env, Val>;
    /// Error type returned if the migration fails.
    /// It must implement the `Into<ContractError>` trait if migration is implemented with the `#[derive(Upgradable)]` macro.     
    type Error;

    /// Migrates contract state after upgrading the contract code.
    fn __migrate(_env: &Env, _migration_data: Self::MigrationData) -> Result<(), Self::Error>;
}

/// This function checks that the caller can authenticate as the owner of the contract,
/// then upgrades the contract to a new WASM hash and prepares it for migration.
pub fn upgrade<T: OwnableInterface>(env: &Env, new_wasm_hash: BytesN<32>) {
    T::owner(env).require_auth();

    env.deployer().update_current_contract_wasm(new_wasm_hash);
    start_migration(env);
}

/// Migrate the contract to a new version after an upgrade.
///
/// This function checks that the caller can authenticate as the owner of the contract,
/// then runs the custom_migration and finalizes the migration.
/// An event is emitted when the migration, and with it the overall upgrade, is complete.
/// Migration can only be run once, after the [`upgrade`] function has been called.
pub fn migrate<T: CustomMigratableInterface>(
    env: &Env,
    migration_data: T::MigrationData,
) -> Result<(), MigrationError<T::Error>> {
    T::owner(env).require_auth();

    ensure_is_migrating::<T>(env)?;

    custom_migrate::<T>(env, migration_data)?;
    complete_migration(env);

    UpgradedEvent {
        version: T::version(env),
    }
    .emit(env);

    Ok(())
}

fn start_migration(env: &Env) {
    env.storage()
        .instance()
        .set(&storage::migrating::DataKey::Interfaces_Migrating, &());
}

fn ensure_is_migrating<T: CustomMigratableInterface>(
    env: &Env,
) -> Result<(), MigrationError<T::Error>> {
    ensure!(
        env.storage()
            .instance()
            .has(&storage::migrating::DataKey::Interfaces_Migrating),
        MigrationError::NotAllowed
    );

    Ok(())
}

fn custom_migrate<T: CustomMigratableInterface>(
    env: &Env,
    migration_data: T::MigrationData,
) -> Result<(), MigrationError<T::Error>> {
    T::__migrate(env, migration_data).map_err(MigrationError::ExecutionFailed)
}

fn complete_migration(env: &Env) {
    env.storage()
        .instance()
        .remove(&storage::migrating::DataKey::Interfaces_Migrating);
}

#[derive(Clone, Debug, PartialEq, Eq, IntoEvent)]
pub struct UpgradedEvent {
    #[data]
    version: String,
}

pub enum MigrationError<T> {
    NotAllowed,
    ExecutionFailed(T),
}

#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, BytesN, Env, String};

    use crate::interfaces::testdata::{ContractClient, ContractNonTrivialClient, MigrationData};
    use crate::interfaces::upgradable::UpgradedEvent;
    use crate::interfaces::{testdata, upgradable};
    use crate::{assert_auth, assert_auth_err, events};

    const WASM: &[u8] = include_bytes!("testdata/contract_trivial_migration.wasm");
    const WASM_NON_TRIVIAL: &[u8] = include_bytes!("testdata/contract_non_trivial_migration.wasm");

    fn prepare_client_and_bytecode(
        env: &Env,
        owner: Option<Address>,
    ) -> (ContractClient, BytesN<32>) {
        let operator = Address::generate(env);
        let contract_id = env.register(testdata::Contract, (owner, operator));
        let hash = env.deployer().upload_contract_wasm(WASM);
        let client = ContractClient::new(env, &contract_id);
        (client, hash)
    }

    #[test]
    fn upgrade_fails_if_owner_not_set() {
        let env = Env::default();
        let (client, hash) = prepare_client_and_bytecode(&env, None);

        assert!(client.try_upgrade(&hash).is_err());
    }

    #[test]
    fn upgrade_fails_if_caller_not_authenticated() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner));

        assert!(client.try_upgrade(&hash).is_err());
    }

    #[test]
    fn upgrade_fails_if_called_by_non_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner));

        assert_auth_err!(Address::generate(&env), client.upgrade(&hash));
    }

    #[test]
    fn upgrade_succeeds_if_owner_is_authenticated() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_auth!(owner, client.upgrade(&hash));
    }

    #[test]
    fn migrate_fails_if_caller_not_authenticated() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_auth!(owner, client.upgrade(&hash));
        assert!(client.try_migrate(&()).is_err());
    }

    #[test]
    fn migrate_fails_if_called_by_non_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_auth!(owner, client.upgrade(&hash));
        assert_auth_err!(Address::generate(&env), client.migrate(&()));
    }

    #[test]
    fn migrate_fails_if_not_called_after_upgrade() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, _) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_auth_err!(owner, client.migrate(&()));
    }

    #[test]
    fn trivial_migrate_succeeds_if_owner_is_authenticated_and_called_after_upgrade() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_auth!(owner, client.upgrade(&hash));

        assert!(client.migration_data().is_none());

        assert_auth!(owner, client.migrate(&()));
        goldie::assert!(events::fmt_last_emitted_event::<UpgradedEvent>(&env));

        assert_eq!(
            client.migration_data(),
            Some(String::from_str(&env, "migrated"))
        );
    }

    #[test]
    fn non_trivial_migrate_succeeds_if_owner_is_authenticated_and_called_after_upgrade() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let operator = Address::generate(&env);
        let contract_id = env.register(testdata::Contract, (owner.clone(), operator));
        let hash = env.deployer().upload_contract_wasm(WASM_NON_TRIVIAL);
        let client = ContractNonTrivialClient::new(&env, &contract_id);

        assert_auth!(owner, client.upgrade(&hash));

        assert!(client.migration_data().is_none());

        let data = MigrationData {
            data1: String::from_str(&env, "migrated_non_trivial"),
            data2: true,
            data3: 42,
        };

        assert_auth!(owner, client.migrate(&data));
        goldie::assert!(events::fmt_last_emitted_event::<UpgradedEvent>(&env));

        assert_eq!(client.migration_data(), Some(data.data1));
    }

    // Because migration happens on a contract loaded from WASM, code coverage analysis doesn't recognize
    // the migration code as covered. This test repeats the migration test with mocked setup
    #[test]
    fn simulate_migration_for_code_coverage() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(testdata::Contract, (Some(owner.clone()), None::<Address>));

        env.as_contract(&contract_id, || {
            upgradable::start_migration(&env);
        });

        let client = ContractClient::new(&env, &contract_id);
        assert_auth!(owner, client.migrate(&()));
    }

    #[test]
    fn migrate_fails_if_called_twice() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_auth!(owner, client.upgrade(&hash));
        assert_auth!(owner, client.migrate(&()));
        assert_auth_err!(owner, client.migrate(&()));
    }
}
