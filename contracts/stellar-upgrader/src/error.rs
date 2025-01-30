use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    SameVersion = 1,
    UnexpectedNewVersion = 2,
}
