use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};

use crate as stellar_axelar_std;

mod operatable {
    use stellar_axelar_std::interfaces::OperatableClient;
    use stellar_axelar_std::{assert_auth, assert_auth_err, only_operator};
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

        #[only_operator]
        pub fn operator_function(env: &Env) {}
    }

    #[test]
    fn operator_function_succeeds_with_correct_operator() {
        let env = Env::default();
        let operator = Address::generate(&env);
        let contract_id = env.register(Contract, (operator.clone(),));
        let client = ContractClient::new(&env, &contract_id);

        assert_auth!(operator, client.operator_function());
    }

    #[test]
    fn operator_function_fails_with_incorrect_operator() {
        let env = Env::default();
        let operator = Address::generate(&env);
        let non_operator = Address::generate(&env);
        let contract_id = env.register(Contract, (operator,));
        let client = ContractClient::new(&env, &contract_id);

        assert_auth_err!(non_operator, client.operator_function());
    }

    #[test]
    fn contract_operatorship_transfer_succeeds() {
        let env = Env::default();
        let operator = Address::generate(&env);
        let contract_id = env.register(Contract, (operator.clone(),));
        let client = OperatableClient::new(&env, &contract_id);
        assert_eq!(operator, client.operator());

        let new_operator = Address::generate(&env);
        assert_auth!(operator, client.transfer_operatorship(&new_operator));
        assert_eq!(new_operator, client.operator());
    }
}

mod ownable {
    use stellar_axelar_std::interfaces::OwnableClient;
    use stellar_axelar_std::{assert_auth, assert_auth_err, only_owner};
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

        #[only_owner]
        pub fn owner_function(env: &Env) {}
    }

    #[test]
    fn owner_function_succeeds_with_correct_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner.clone(),));
        let client = ContractClient::new(&env, &contract_id);

        assert_auth!(owner, client.owner_function());
    }

    #[test]
    fn owner_function_fails_with_incorrect_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let non_owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner,));
        let client = ContractClient::new(&env, &contract_id);

        assert_auth_err!(non_owner, client.owner_function());
    }

    #[test]
    fn contract_ownership_transfer_succeeds() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner.clone(),));
        let client = OwnableClient::new(&env, &contract_id);
        assert_eq!(owner, client.owner());

        let new_owner = Address::generate(&env);
        assert_auth!(owner, client.transfer_ownership(&new_owner));
        assert_eq!(new_owner, client.owner());
    }
}
mod pausable {
    use stellar_axelar_std::assert_auth;
    use stellar_axelar_std::interfaces::PausableClient;
    use stellar_axelar_std_derive::{Ownable, Pausable};

    use super::*;

    #[contract]
    #[derive(Ownable, Pausable)]
    pub struct Contract;

    #[contractimpl]
    impl Contract {
        pub fn __constructor(env: &Env, owner: Address) {
            stellar_axelar_std::interfaces::set_owner(env, &owner);
        }
    }

    #[test]
    fn contract_pause_unpause_succeeds() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner.clone(),));
        let client = PausableClient::new(&env, &contract_id);

        assert!(!client.paused());

        assert_auth!(owner, client.pause());
        assert!(client.paused());

        assert_auth!(owner, client.unpause());
        assert!(!client.paused());
    }
}

mod upgradable {
    use stellar_axelar_std::assert_auth;
    use stellar_axelar_std_derive::{Ownable, Upgradable};

    use super::*;
    use crate::interfaces::CustomMigratableInterface;
    use crate::std::string::ToString;
    use crate::tests::testdata;

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

    impl CustomMigratableInterface for Contract {
        type MigrationData = ();
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
