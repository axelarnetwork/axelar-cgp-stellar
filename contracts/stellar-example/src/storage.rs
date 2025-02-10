use soroban_sdk::Address;
use stellar_axelar_std::contractstorage;

#[contractstorage]
#[derive(Clone, Debug)]
enum DataKey {
    #[instance]
    #[value(Address)]
    Gateway,

    #[instance]
    #[value(Address)]
    GasService,

    #[instance]
    #[value(Address)]
    InterchainTokenService,

    #[instance]
    #[status]
    Paused,
}
