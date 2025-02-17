use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address};
use stellar_axelar_std::interfaces::CustomMigratableInterface;
use stellar_axelar_std::{assert_auth, assert_err};

use crate::error::ContractError;
use crate::tests::testutils::{setup_env, TestConfig};
use crate::AxelarOperators;

const NEW_WASM: &[u8] = include_bytes!("testdata/stellar_axelar_operators.optimized.wasm");

#[test]
fn migrate_succeeds() {
    let TestConfig {
        env, owner, client, ..
    } = setup_env();

    let new_wasm_hash = env.deployer().upload_contract_wasm(NEW_WASM);

    let operator = Address::generate(&env);

    assert_auth!(owner, client.add_operator(&operator));
    assert!(client.is_operator(&operator));

    assert_auth!(owner, client.upgrade(&new_wasm_hash));

    let migration_data = vec![&env, operator.clone()];

    assert_auth!(owner, client.migrate(&migration_data));

    assert!(
        client.is_operator(&operator),
        "Operator should still exist after migration"
    );
}

#[test]
fn migrate_fails_when_account_is_not_operator() {
    let TestConfig {
        env, owner, client, ..
    } = setup_env();

    let new_wasm_hash = env.deployer().upload_contract_wasm(NEW_WASM);
    let non_operator = Address::generate(&env);

    assert_auth!(owner, client.upgrade(&new_wasm_hash));

    let migration_data = vec![&env, non_operator];

    assert_err!(
        env.as_contract(&client.address, || {
            <AxelarOperators as CustomMigratableInterface>::__migrate(&env, migration_data)
        }),
        ContractError::NotAnOperator
    );
}

#[test]
fn migrate_succeeds_with_multiple_operators() {
    let TestConfig {
        env, owner, client, ..
    } = setup_env();

    let new_wasm_hash = env.deployer().upload_contract_wasm(NEW_WASM);

    let operator1 = Address::generate(&env);
    let operator2 = Address::generate(&env);

    assert_auth!(owner, client.add_operator(&operator1));
    assert_auth!(owner, client.add_operator(&operator2));

    assert!(client.is_operator(&operator1));
    assert!(client.is_operator(&operator2));

    assert_auth!(owner, client.upgrade(&new_wasm_hash));

    let migration_data = vec![&env, operator1.clone(), operator2.clone()];
    assert_auth!(owner, client.migrate(&migration_data));

    assert!(client.is_operator(&operator1));
    assert!(client.is_operator(&operator2));
}
