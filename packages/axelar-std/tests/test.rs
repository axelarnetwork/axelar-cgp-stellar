use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};

mod testdata;
mod operatable {
    use stellar_axelar_std::assert_auth;
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
        assert_auth!(operator, client.transfer_operatorship(&new_operator));
        assert_eq!(new_operator, client.operator());
    }
}

mod ownable {
    use stellar_axelar_std::assert_auth;
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
        assert_auth!(owner, client.transfer_ownership(&new_owner));
        assert_eq!(new_owner, client.owner());
    }
}

mod upgradable {
    use stellar_axelar_std::assert_auth;
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

        assert_auth!(owner, client.upgrade(&new_wasm_hash));

        let client = testdata::ContractClient::new(env, &contract_id);
        assert_auth!(owner, client.migrate(&()));
    }
}

mod into_event {
    use soroban_sdk::{Address, String};
    use stellar_axelar_std::events::{fmt_last_emitted_event, Event};
    use stellar_axelar_std_derive::IntoEvent;

    use super::*;

    #[contract]
    pub struct IntoEventTest;

    #[contractimpl]
    impl IntoEventTest {
        pub fn __constructor(env: &Env, owner: Address) {
            stellar_axelar_std::interfaces::set_owner(env, &owner);
        }

        pub fn transfer_emission(env: &Env, from: Address, to: Address, amount: String) {
            TransferredEvent { from, to, amount }.emit(env);
        }

        pub fn multi_data_emission(
            env: &Env,
            topic1: String,
            data1: String,
            topic2: Address,
            data2: String,
        ) {
            MultiDataEvent {
                topic1,
                data1,
                topic2,
                data2,
            }
            .emit(env);
        }
    }

    #[derive(Debug, PartialEq, Eq, IntoEvent)]
    struct TransferredEvent {
        from: Address,
        to: Address,
        #[data]
        amount: String,
    }

    #[test]
    fn event_transfer_emission_succeeds() {
        let env = Env::default();

        let owner = Address::generate(&env);
        let contract_id = env.register(IntoEventTest, (owner,));
        let client = IntoEventTestClient::new(&env, &contract_id);

        let from = Address::generate(&env);
        let to = Address::generate(&env);
        let amount = String::from_str(&env, "100");

        client.transfer_emission(&from, &to, &amount);

        TransferredEvent { from, to, amount }.emit(&env);

        goldie::assert!(fmt_last_emitted_event::<TransferredEvent>(&env));
    }

    #[derive(Debug, PartialEq, Eq, IntoEvent)]
    #[event_name("emitted_data")]
    struct MultiDataEvent {
        topic1: String,
        #[data]
        data1: String,
        topic2: Address,
        #[data]
        data2: String,
    }

    #[test]
    fn event_custom_multiple_topics_and_data_succeeds() {
        let env = Env::default();

        let owner = Address::generate(&env);
        let contract_id = env.register(IntoEventTest, (owner,));
        let client = IntoEventTestClient::new(&env, &contract_id);

        let topic1 = String::from_str(&env, "topic-1");
        let data1 = String::from_str(&env, "data-1");
        let topic2 = Address::generate(&env);
        let data2 = String::from_str(&env, "data-2");

        client.multi_data_emission(&topic1, &data1, &topic2, &data2);

        MultiDataEvent {
            topic1: topic1.clone(),
            data1: data1.clone(),
            topic2: topic2.clone(),
            data2: data2.clone(),
        }
        .emit(&env);

        goldie::assert!(fmt_last_emitted_event::<MultiDataEvent>(&env));
    }
}
