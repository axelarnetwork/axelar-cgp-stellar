use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    OperatorAlreadyAdded = 2,
    NotAnOperator = 3,
}
