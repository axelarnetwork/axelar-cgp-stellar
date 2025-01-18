use soroban_sdk::{Address, Bytes, Env, String};

const ZERO_ADDRESS: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
const STELLAR_ADDRESS_LEN: usize = ZERO_ADDRESS.len();

pub trait AddressExt {
    fn zero(env: &Env) -> Address;
    fn to_string_bytes(&self, env: &Env) -> Bytes;
}

impl AddressExt for Address {
    /// Returns Stellar's "dead" address, represented by the constant `ZERO_ADDRESS`.
    /// # Reference
    /// - Stellar [GitHub](https://github.com/stellar/js-stellar-base/blob/master/test/unit/address_test.js)
    fn zero(env: &Env) -> Address {
        Self::from_string(&String::from_str(env, ZERO_ADDRESS))
    }

    // Converts Stellar address (soroban_sdk::Address) to soroban_sdk::Bytes
    fn to_string_bytes(&self, env: &Env) -> Bytes {
        let mut address_string_bytes = [0u8; STELLAR_ADDRESS_LEN];
        self.to_string().copy_into_slice(&mut address_string_bytes);
        Bytes::from_slice(env, &address_string_bytes)
    }
}
