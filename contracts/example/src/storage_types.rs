use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Gateway,
    GasService,
    InterchainTokenService,
}
