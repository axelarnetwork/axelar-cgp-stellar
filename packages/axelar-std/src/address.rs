use soroban_sdk::{Address, Env, String};

const ZERO_ADDRESS: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";

pub trait AddressExt {
    fn zero(env: &Env) -> Address;
}

impl AddressExt for Address {
    /// Returns Stellar's "dead" address, represented by the constant `ZERO_ADDRESS`.
    /// # Reference
    /// - Stellar [GitHub](https://github.com/stellar/js-stellar-base/blob/master/test/unit/address_test.js)
    fn zero(env: &Env) -> Address {
        Self::from_string(&String::from_str(env, ZERO_ADDRESS))
    }
}
