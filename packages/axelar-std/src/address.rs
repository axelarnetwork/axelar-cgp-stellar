use soroban_sdk::{Address, Bytes, Env, String};

const ZERO_ADDRESS: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
const STELLAR_ADDRESS_LEN: usize = 56;

pub trait AddressExt {
    fn zero(env: &Env) -> Address;
    fn to_bytes(&self, env: &Env) -> Bytes;
}

impl AddressExt for Address {
    /// Returns Stellar's "dead" address, represented by the constant `ZERO_ADDRESS`.
    /// # Reference
    /// - Stellar [GitHub](https://github.com/stellar/js-stellar-base/blob/master/test/unit/address_test.js)
    fn zero(env: &Env) -> Address {
        Self::from_string(&String::from_str(env, ZERO_ADDRESS))
    }

    // Converts Stellar address (soroban_sdk::Address) to soroban_sdk::Bytes
    fn to_bytes(&self, env: &Env) -> Bytes {
        let address_str = self.to_string();
        let mut str_bytes = [0u8; STELLAR_ADDRESS_LEN];
        address_str.copy_into_slice(&mut str_bytes);
        Bytes::from_slice(env, &str_bytes)
    }
}
