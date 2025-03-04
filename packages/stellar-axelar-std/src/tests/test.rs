use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};

use crate as stellar_axelar_std;

mod upgradable {
    use stellar_axelar_std::assert_auth;
    use stellar_axelar_std_derive::{Ownable, Upgradable};

    use super::*;
    use crate::std::string::ToString;
    use crate::tests::testdata;

    #[contracterror]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ContractError {
        MigrationNotAllowed = 1,
    }

    #[contract]
    #[derive(Ownable, Upgradable)]
    pub struct Contract;

    #[contractimpl]
    impl Contract {
        pub fn __constructor(env: &Env, owner: Address) {
            stellar_axelar_std::interfaces::set_owner(env, &owner);
        }
    }

    const UPGRADED_WASM: &[u8] = include_bytes!("testdata/contract.wasm");

    #[test]
    fn contract_version_exists() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner,));
        let client = ContractClient::new(&env, &contract_id);
        let contract_version = client.version();
        assert_eq!(contract_version.to_string(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn contract_upgrade_succeeds() {
        let env = &Env::default();
        let owner = Address::generate(env);
        let contract_id = env.register(Contract, (owner.clone(),));
        let client = ContractClient::new(env, &contract_id);
        let new_wasm_hash = env.deployer().upload_contract_wasm(UPGRADED_WASM);

        assert_auth!(owner, client.upgrade(&new_wasm_hash));

        let client = testdata::ContractClient::new(env, &contract_id);
        assert_auth!(owner, client.migrate(&()));
    }
}
