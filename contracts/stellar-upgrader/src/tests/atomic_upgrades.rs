use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env, String};
use stellar_axelar_std::{assert_contract_err, mock_auth};

use super::utils::{DummyContract, DummyContractClient};
use crate::error::ContractError;
use crate::tests::utils;
use crate::{Upgrader, UpgraderClient};

const WASM_AFTER_UPGRADE: &[u8] = include_bytes!("testdata/dummy.wasm");

#[test]
fn upgrade_and_migrate_are_atomic() {
    let TestFixture {
        env,
        upgrader_address,
        contract_owner: owner,
        contract_address,
        hash_after_upgrade,
        expected_data,
        expected_version,
    } = setup_contracts_and_call_args();

    let dummy_client = DummyContractClient::new(&env, &contract_address);
    let original_version: String = dummy_client.version();
    assert_eq!(original_version, String::from_str(&env, "0.1.0"));

    let upgrader = UpgraderClient::new(&env, &upgrader_address);

    let upgrade_auth = mock_auth!(owner, dummy_client.upgrade(hash_after_upgrade));
    let migrate_auth = mock_auth!(owner, dummy_client.migrate(expected_data));

    upgrader.mock_auths(&[upgrade_auth, migrate_auth]).upgrade(
        &contract_address,
        &expected_version,
        &hash_after_upgrade,
        &soroban_sdk::vec![&env, expected_data.to_val()],
    );

    // ensure new version is set correctly
    let upgraded_version: String = dummy_client.version();
    assert_eq!(upgraded_version, expected_version);

    // ensure migration was successful
    env.as_contract(&contract_address, || {
        let data = utils::storage::data(&env);
        assert_eq!(data, expected_data);
    });
}

#[test]
fn upgrade_fails_if_upgrading_to_the_same_version() {
    let env = Env::default();

    let upgrader_address = env.register(Upgrader, ());

    let contract_owner = Address::generate(&env);
    let contract_address = env.register(DummyContract, (&contract_owner,));
    let dummy_hash = BytesN::from_array(&env, &[1u8; 32]);
    let dummy_data = String::from_str(&env, "");
    let original_version = String::from_str(&env, "0.1.0");

    let upgrader = UpgraderClient::new(&env, &upgrader_address);

    assert_contract_err!(
        upgrader
            .mock_all_auths_allowing_non_root_auth()
            .try_upgrade(
                &contract_address,
                &original_version,
                &dummy_hash,
                &soroban_sdk::vec![&env, dummy_data.to_val()],
            ),
        ContractError::SameVersion
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn upgrade_fails_if_caller_is_authenticated_but_not_owner() {
    let TestFixture {
        env,
        upgrader_address,
        contract_address,
        hash_after_upgrade,
        expected_data,
        expected_version,
        ..
    } = setup_contracts_and_call_args();

    let dummy_client = DummyContractClient::new(&env, &contract_address);
    let upgrader = UpgraderClient::new(&env, &upgrader_address);

    // add the caller to the set of authenticated addresses
    let caller = Address::generate(&env);

    let upgrade_auth = mock_auth!(caller, dummy_client.upgrade(hash_after_upgrade));
    let migrate_auth = mock_auth!(caller, dummy_client.migrate(expected_data));

    // should panic: caller is authenticated, but not the owner
    upgrader.mock_auths(&[upgrade_auth, migrate_auth]).upgrade(
        &contract_address,
        &expected_version,
        &hash_after_upgrade,
        &soroban_sdk::vec![&env, expected_data.to_val()],
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn upgrade_fails_if_correct_owner_is_not_authenticated_for_full_invocation_tree() {
    let TestFixture {
        env,
        upgrader_address,
        contract_owner: owner,
        contract_address,
        hash_after_upgrade,
        expected_data,
        expected_version,
    } = setup_contracts_and_call_args();
    let dummy_client = DummyContractClient::new(&env, &contract_address);
    let upgrader = UpgraderClient::new(&env, &upgrader_address);

    // add the caller to the set of authenticated addresses
    let caller = Address::generate(&env);

    let upgrade_auth = mock_auth!(owner, dummy_client.upgrade(hash_after_upgrade));
    let migrate_auth = mock_auth!(caller, dummy_client.migrate(expected_data));

    // only add the owner to the set of authenticated addresses for the upgrade function, and the caller for the migrate function
    upgrader.mock_auths(&[upgrade_auth, migrate_auth]).upgrade(
        &contract_address,
        &expected_version,
        &hash_after_upgrade,
        &soroban_sdk::vec![&env, expected_data.to_val()],
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn upgrade_fails_if_nobody_is_authenticated() {
    let TestFixture {
        env,
        upgrader_address,
        contract_address,
        hash_after_upgrade,
        expected_data,
        expected_version,
        ..
    } = setup_contracts_and_call_args();

    UpgraderClient::new(&env, &upgrader_address).upgrade(
        &contract_address,
        &expected_version,
        &hash_after_upgrade,
        &soroban_sdk::vec![&env, expected_data.to_val()],
    );
}

struct TestFixture {
    env: Env,
    upgrader_address: Address,
    contract_owner: Address,
    contract_address: Address,
    hash_after_upgrade: BytesN<32>,
    expected_data: String,
    expected_version: String,
}

fn setup_contracts_and_call_args() -> TestFixture {
    let env = Env::default();

    let upgrader_address = env.register(Upgrader, ());

    let contract_owner = Address::generate(&env);
    let contract_address = env.register(DummyContract, (&contract_owner,));

    let hash_after_upgrade = env.deployer().upload_contract_wasm(WASM_AFTER_UPGRADE);
    let expected_data = String::from_str(&env, "migration successful");
    let expected_version = String::from_str(&env, "0.2.0");

    TestFixture {
        env,
        upgrader_address,
        contract_owner,
        contract_address,
        hash_after_upgrade,
        expected_data,
        expected_version,
    }
}
