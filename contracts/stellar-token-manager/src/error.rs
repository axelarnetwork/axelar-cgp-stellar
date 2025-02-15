use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
}
