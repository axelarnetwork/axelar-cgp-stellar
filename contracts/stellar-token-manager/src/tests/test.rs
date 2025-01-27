#![cfg(test)]
extern crate std;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, vec, Address, Env, IntoVal, Symbol, Val,
    Vec,
};
use stellar_axelar_std::events::Event;
use stellar_axelar_std::{assert_auth, assert_auth_err, IntoEvent};

use crate::{TokenManager, TokenManagerClient};

#[contract]
pub struct TestTarget;

#[contracterror]
pub enum TestTargetError {
    TestError = 1,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct ExecutedEvent {
    pub value: u32,
}

#[contractimpl]
impl TestTarget {
    pub fn method(env: &Env, value: u32) {
        ExecutedEvent { value }.emit(env);
    }

    pub fn failing(_env: &Env) {
        panic!("This method should fail");
    }

    pub const fn failing_with_error(_env: &Env) -> Result<Val, TestTargetError> {
        Err(TestTargetError::TestError)
    }
}

fn setup<'a>() -> (Env, TokenManagerClient<'a>, Address) {
    let env = Env::default();

    let owner = Address::generate(&env);
    let contract_id = env.register(TokenManager, (owner,));
    let client = TokenManagerClient::new(&env, &contract_id);

    let target_id = env.register(TestTarget, ());

    (env, client, target_id)
}

#[test]
fn register_contract_succeeds() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let contract_id = env.register(TokenManager, (owner.clone(),));
    let client = TokenManagerClient::new(&env, &contract_id);

    assert_eq!(client.owner(), owner);
}

#[test]
fn execute_succeeds() {
    let (env, client, target) = setup();

    assert_auth!(
        client.owner(),
        client.execute(
            &target,
            &symbol_short!("method"),
            &vec![&env, IntoVal::<_, Val>::into_val(&42u32, &env)],
        )
    );
}

#[test]
fn execute_fails_when_not_owner() {
    let (env, client, target) = setup();
    let not_owner = Address::generate(&env);

    assert_auth_err!(
        not_owner,
        client.execute(
            &target,
            &symbol_short!("method"),
            &vec![&env, IntoVal::<_, Val>::into_val(&42u32, &env)],
        )
    );
}

#[test]
#[should_panic]
fn execute_fails_when_target_panics() {
    let (env, client, target) = setup();

    assert_auth!(
        client.owner(),
        client.execute(
            &target,
            &Symbol::new(&env, "failing"),
            &Vec::<Val>::new(&env),
        )
    );
}

#[test]
fn execute_fails_when_target_returns_error() {
    let (env, client, target) = setup();

    assert_auth_err!(
        client.owner(),
        client.execute(
            &target,
            &Symbol::new(&env, "failing_with_error"),
            &Vec::<Val>::new(&env),
        )
    );
}
