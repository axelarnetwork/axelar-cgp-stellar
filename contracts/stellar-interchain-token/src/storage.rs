use soroban_sdk::{contracttype, Address, BytesN};
use stellar_axelar_std::contractstorage;

#[contracttype]
#[derive(Clone)]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

/// Do not use the symbol `METADATA` as a key as it is reserved for token metadata.
#[contractstorage]
enum DataKey {
    #[temporary]
    #[value(AllowanceValue)]
    Allowance { key: AllowanceDataKey },

    #[persistent]
    #[value(i128)]
    Balance { address: Address },

    #[instance]
    #[status]
    Minter { minter: Address },

    #[instance]
    #[value(BytesN<32>)]
    TokenId,
}
