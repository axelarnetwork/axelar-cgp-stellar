#![cfg(test)]
extern crate std;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, vec, Address, Env, Symbol, Val, Vec,
};
use stellar_axelar_std::events::{fmt_last_emitted_event, Event};
use stellar_axelar_std::{assert_auth, assert_auth_err, assert_contract_err, IntoEvent};

use crate::error::ContractError;
use crate::event::{OperatorAddedEvent, OperatorRemovedEvent};
use crate::{AxelarOperators, AxelarOperatorsClient};

#[contract]
pub struct TestTarget;

#[contracterror]
pub enum TestTargetError {
    TestError = 1,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct ExecutedEvent;

#[contractimpl]
impl TestTarget {
    pub fn method(env: &Env) {
        ExecutedEvent.emit(env);
    }

    pub fn failing(_env: &Env) {
        panic!("This method should fail");
    }

    pub const fn failing_with_error(_env: &Env) -> Result<Val, TestTargetError> {
        Err(TestTargetError::TestError)
    }
}

struct TestConfig<'a> {
    env: Env,
    owner: Address,
    client: AxelarOperatorsClient<'a>,
    target_id: Address,
}

fn setup_env<'a>() -> TestConfig<'a> {
    let env = Env::default();

    let owner = Address::generate(&env);
    let contract_id = env.register(AxelarOperators, (&owner,));
    let client = AxelarOperatorsClient::new(&env, &contract_id);

    let target_id = env.register(TestTarget, ());

    TestConfig {
        env,
        owner,
        client,
        target_id,
    }
}

#[test]
fn register_operators() {
    let TestConfig { owner, client, .. } = setup_env();

    assert_eq!(client.owner(), owner);
}

#[test]
fn add_operator_succeeds() {
    let TestConfig { env, client, .. } = setup_env();
    let operator = Address::generate(&env);

    assert!(!client.is_operator(&operator));

    assert_auth!(client.owner(), client.add_operator(&operator));

    goldie::assert!(fmt_last_emitted_event::<OperatorAddedEvent>(&env));

    assert!(client.is_operator(&operator));
}

#[test]
fn add_operator_fails_when_already_added() {
    let TestConfig { env, client, .. } = setup_env();
    let operator = Address::generate(&env);

    assert_auth!(&client.owner(), client.add_operator(&operator));

    assert_contract_err!(
        client.mock_all_auths().try_add_operator(&operator),
        ContractError::OperatorAlreadyAdded
    );
}

#[test]
fn remove_operator_succeeds() {
    let TestConfig { env, client, .. } = setup_env();
    let operator = Address::generate(&env);

    assert_auth!(client.owner(), client.add_operator(&operator));

    assert!(client.is_operator(&operator));

    assert_auth!(client.owner(), client.remove_operator(&operator));

    goldie::assert!(fmt_last_emitted_event::<OperatorRemovedEvent>(&env));

    assert!(!client.is_operator(&operator));
}

#[test]
fn remove_operator_fails_when_not_an_operator() {
    let TestConfig { env, client, .. } = setup_env();
    let operator = Address::generate(&env);

    assert_contract_err!(
        client.mock_all_auths().try_remove_operator(&operator),
        ContractError::NotAnOperator
    );
}

#[test]
fn execute_succeeds() {
    let TestConfig {
        env,
        client,
        target_id,
        ..
    } = setup_env();
    let operator = Address::generate(&env);

    assert_auth!(&client.owner(), client.add_operator(&operator));

    assert_auth!(
        operator,
        client.execute(
            &operator,
            &target_id,
            &symbol_short!("method"),
            &Vec::<Val>::new(&env),
        )
    );

    goldie::assert!(fmt_last_emitted_event::<ExecutedEvent>(&env));
}

#[test]
fn execute_fails_when_not_an_operator() {
    let TestConfig { env, client, .. } = setup_env();

    assert_contract_err!(
        client.mock_all_auths().try_execute(
            &client.owner(),
            &client.address,
            &symbol_short!("method"),
            &Vec::new(&env)
        ),
        ContractError::NotAnOperator
    );
}

#[test]
#[should_panic]
fn execute_fails_when_target_panics() {
    let TestConfig {
        env,
        client,
        target_id,
        ..
    } = setup_env();
    let operator = Address::generate(&env);

    assert_auth!(&client.owner(), client.add_operator(&operator));

    assert_auth!(
        operator,
        client.execute(
            &operator,
            &target_id,
            &Symbol::new(&env, "failing"),
            &Vec::<Val>::new(&env),
        )
    );
}

#[test]
fn execute_fails_when_target_returns_error() {
    let TestConfig {
        env,
        client,
        target_id,
        ..
    } = setup_env();
    let operator = Address::generate(&env);

    assert_auth!(&client.owner(), client.add_operator(&operator));

    assert_auth_err!(
        operator,
        client.execute(
            &operator,
            &target_id,
            &Symbol::new(&env, "failing"),
            &Vec::<Val>::new(&env),
        )
    );
}

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
