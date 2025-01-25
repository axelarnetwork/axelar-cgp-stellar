use soroban_sdk::{Address, BytesN, Env, String, Val};

use crate::error::ContractError;

pub trait UpgraderInterface {
    /// Upgrades and migrates a contract atomically to a new version using the provided WASM hash and migration data.
    ///
    /// # Arguments
    /// * `contract_address` - The address of the contract to be upgraded.
    /// * `new_version` - The new version string for the contract.
    /// * `new_wasm_hash` - The hash of the new WASM code for the contract.
    /// * `migration_data` - The data to be used during the migration process.
    ///
    /// # Errors
    /// - [`ContractError::SameVersion`]: If the new version is the same as the current version.
    /// - [`ContractError::UnexpectedNewVersion`]: If the contract version after the upgrade does not match the expected new version.
    fn upgrade(
        env: Env,
        contract_address: Address,
        new_version: String,
        new_wasm_hash: BytesN<32>,
        migration_data: soroban_sdk::Vec<Val>,
    ) -> Result<(), ContractError>;
}
