mod contract_non_trivial_migration;
mod contract_trivial_migration;

pub use contract_non_trivial_migration::{ContractNonTrivialClient, MigrationData};
pub use contract_trivial_migration::{Contract, ContractClient};
