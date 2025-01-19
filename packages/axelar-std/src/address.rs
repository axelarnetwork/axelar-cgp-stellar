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

    /// Converts Stellar address to a string represented as bytes
    fn to_string_bytes(&self, env: &Env) -> Bytes {
        let mut address_string_bytes = [0u8; STELLAR_ADDRESS_LEN];
        self.to_string().copy_into_slice(&mut address_string_bytes);
        Bytes::from_slice(env, &address_string_bytes)
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String};

    use super::AddressExt;

    fn address_to_bytes_and_back(env: &Env, address: &Address) -> Address {
        let address_bytes = address.to_string_bytes(env);
        Address::from_string_bytes(&address_bytes)
    }

    #[test]
    fn generated_address_to_string_bytes() {
        let env = Env::default();
        let original_address = Address::generate(&env);
        let converted_address = address_to_bytes_and_back(&env, &original_address);
        assert_eq!(original_address, converted_address);
    }

    #[test]
    fn zero_address_to_string_bytes() {
        let env = Env::default();
        let zero_address = Address::zero(&env);
        let converted_address = address_to_bytes_and_back(&env, &zero_address);
        assert_eq!(zero_address, converted_address);
    }

    #[test]
    fn account_keys_to_string_bytes() {
        let env = Env::default();
        let account_keys: [&str; 3] = [
            "GC7OHFPWPSWXL4HMN6TXAG54MTZSMJIASWHO6KVRQNHNCXEAHWDSGGC3",
            "GC65CUPW2IMTJJY6CII7F3OBPVG4YGASEPBBLM4V3LBKX62P6LA24OFV",
            "GCXQYWGBS5QIXSURFRL3EQIIY556F2TKUYBNWZKEUPNJAHEVIGGPWX3Y",
        ];

        for key in &account_keys {
            let original_address = Address::from_string(&String::from_str(&env, key));
            let converted_address = address_to_bytes_and_back(&env, &original_address);
            assert_eq!(original_address, converted_address);
        }
    }

    #[test]
    fn contract_keys_to_string_bytes() {
        let env = Env::default();
        let contract_keys: [&str; 3] = [
            "CB6743BTQ2TZHTUCRSUFAH2X5ICOZGI6I3UCQBY3VFSSJ7IERGXUM7TX",
            "CCNPLLAHDRCYOY2RGUGBYAEWXEPCSQSZZGYYCQWHEC2KBYBWLVKJAU4D",
            "CD7I2MTBYIQJ6KWR5FLILJLDBJGDV3FTIV24XRIXAEMSR72SRF4AQQCF",
        ];

        for key in &contract_keys {
            let original_address = Address::from_string(&String::from_str(&env, key));
            let converted_address = address_to_bytes_and_back(&env, &original_address);
            assert_eq!(original_address, converted_address);
        }
    }
}
