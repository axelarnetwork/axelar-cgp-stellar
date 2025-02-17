use soroban_sdk::Address;
use stellar_axelar_std::contractstorage;

#[contractstorage]
enum DataKey {
    #[instance]
    #[status]
    Operator { account: Address },
}
