use soroban_sdk::{Address, Bytes, Env, String};

const ZERO_ADDRESS: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
const STELLAR_ADDRESS_LEN: usize = ZERO_ADDRESS.len();

pub trait AddressExt {
    fn zero(env: &Env) -> Address;

    fn to_string_bytes(&self) -> Bytes;

    fn to_raw_bytes(&self) -> [u8; STELLAR_ADDRESS_LEN];
}

impl AddressExt for Address {
    /// Returns Stellar's ["dead"](https://github.com/stellar/js-stellar-base/blob/master/test/unit/address_test.js) address, represented by the constant `ZERO_ADDRESS`.
    fn zero(env: &Env) -> Address {
        Self::from_string(&String::from_str(env, ZERO_ADDRESS))
    }

    /// Converts Stellar address to a string represented as bytes
    fn to_string_bytes(&self) -> Bytes {
        let mut address_string_bytes = [0u8; STELLAR_ADDRESS_LEN];
        self.to_string().copy_into_slice(&mut address_string_bytes);
        Bytes::from_slice(self.env(), &address_string_bytes)
    }

    fn to_raw_bytes(&self) -> [u8; STELLAR_ADDRESS_LEN] {
        let mut address_string_bytes = [0u8; STELLAR_ADDRESS_LEN];
        self.to_string().copy_into_slice(&mut address_string_bytes);
        address_string_bytes
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, String};

    use super::{AddressExt, ZERO_ADDRESS};

    #[test]
    fn zero_address_to_string() {
        let env = &Env::default();
        assert_eq!(
            Address::zero(env).to_string(),
            String::from_str(env, ZERO_ADDRESS)
        );
    }

    #[test]
    fn string_to_address_to_string() {
        let env = &Env::default();
        let cases = [
            ZERO_ADDRESS,
            "GC7OHFPWPSWXL4HMN6TXAG54MTZSMJIASWHO6KVRQNHNCXEAHWDSGGC3",
            "CB6743BTQ2TZHTUCRSUFAH2X5ICOZGI6I3UCQBY3VFSSJ7IERGXUM7TX",
        ]
        .into_iter()
        .map(|s| Bytes::from_slice(env, s.as_bytes()));

        for address_bytes in cases {
            let address = Address::from_string_bytes(&address_bytes);
            assert_eq!(address.to_string_bytes(), address_bytes);
        }
    }

    #[test]
    fn address_to_string_to_address() {
        let env = &Env::default();

        let cases = [
            Address::zero(env),
            Address::generate(env),
            env.register_stellar_asset_contract_v2(Address::generate(env))
                .address(),
        ];

        for address in cases {
            let address_bytes = address.to_string_bytes();
            assert_eq!(Address::from_string_bytes(&address_bytes), address);
        }
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Value, InvalidInput)")]
    fn unsupported_muxed_address_format_fails_on_conversion() {
        let env = &Env::default();

        let unsupported_address =
            "MA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJUAAAAAAAAAAAACJUQ";
        Address::from_string(&String::from_str(env, unsupported_address));
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Value, InvalidInput)")]
    fn unsupported_signed_payload_address_format_fails_on_conversion() {
        let env = &Env::default();

        let unsupported_address = "PA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJUAAAAAOQCAQDAQCQMBYIBEFAWDANBYHRAEISCMKBKFQXDAMRUGY4DUAAAAFGBU";
        Address::from_string(&String::from_str(env, unsupported_address));
    }
}
