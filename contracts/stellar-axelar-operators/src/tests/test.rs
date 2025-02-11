#![cfg(test)]
extern crate std;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Address, Env, Symbol, Val, Vec,
};
use stellar_axelar_std::events::{fmt_last_emitted_event, Event};
use stellar_axelar_std::{assert_auth, assert_auth_err, assert_contract_err, IntoEvent};

use crate::error::ContractError;
use crate::event::{OperatorAddedEvent, OperatorRemovedEvent};
use crate::tests::testutils::{setup_env, TestConfig};

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
