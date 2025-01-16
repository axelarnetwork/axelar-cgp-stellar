use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};

mod testdata;
mod operatable {
    use stellar_axelar_std::assert_invoke_auth_ok;
    use stellar_axelar_std::interfaces::OperatableClient;
    use stellar_axelar_std_derive::Operatable;

    use super::*;

    #[contracterror]
    #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
    #[repr(u32)]
    enum ContractError {
        MigrationNotAllowed = 1,
    }

    #[contract]
    #[derive(Operatable)]
    pub struct Contract;

    #[contractimpl]
    impl Contract {
        pub fn __constructor(env: &Env, operator: Address) {
            stellar_axelar_std::interfaces::set_operator(env, &operator);
        }
    }

    #[test]
    fn contract_operatorship_transfer_succeeds() {
        let env = Env::default();
        let operator = Address::generate(&env);
        let contract_id = env.register(Contract, (operator.clone(),));
        let client = OperatableClient::new(&env, &contract_id);
        assert_eq!(operator, client.operator());

        let new_operator = Address::generate(&env);
        assert_invoke_auth_ok!(operator, client.try_transfer_operatorship(&new_operator));
        assert_eq!(new_operator, client.operator());
    }
}

mod ownable {
    use stellar_axelar_std::assert_invoke_auth_ok;
    use stellar_axelar_std::interfaces::OwnableClient;
    use stellar_axelar_std_derive::Ownable;

    use super::*;

    #[contracterror]
    #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
    #[repr(u32)]
    enum ContractError {
        MigrationNotAllowed = 1,
    }

    #[contract]
    #[derive(Ownable)]
    pub struct Contract;

    #[contractimpl]
    impl Contract {
        pub fn __constructor(env: &Env, owner: Address) {
            stellar_axelar_std::interfaces::set_owner(env, &owner);
        }
    }

    #[test]
    fn contract_ownership_transfer_succeeds() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner.clone(),));
        let client = OwnableClient::new(&env, &contract_id);
        assert_eq!(owner, client.owner());

        let new_owner = Address::generate(&env);
        assert_invoke_auth_ok!(owner, client.try_transfer_ownership(&new_owner));
        assert_eq!(new_owner, client.owner());
    }
}

mod upgradable {
    use stellar_axelar_std::assert_invoke_auth_ok;
    use stellar_axelar_std_derive::{Ownable, Upgradable};

    use super::*;

    #[contracterror]
    #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
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

    impl Contract {
        const fn run_migration(_env: &Env, _migration_data: ()) {}
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

        assert_invoke_auth_ok!(owner, client.try_upgrade(&new_wasm_hash));

        let client = testdata::ContractClient::new(env, &contract_id);
        assert_invoke_auth_ok!(owner, client.try_migrate(&()));
    }
}
