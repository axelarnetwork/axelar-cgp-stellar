#![cfg(any(test, feature = "testutils"))]
extern crate std;

use soroban_sdk::{
    bytes, symbol_short, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _, Events}, vec, xdr::ToXdr, Address, Bytes, BytesN, Env, IntoVal, String, Symbol, TryFromVal, Val, Vec
};

/// Asserts invocation auth of a contract from a single caller.
pub fn assert_invocation(
    env: &Env,
    caller: &Address,
    contract_id: &Address,
    function_name: &str,
    args: Vec<Val>,
) {
    assert_eq!(
        env.auths(),
        std::vec![(
            caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(env, function_name),
                    args,
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

/// Asserts that the event at `event_index` in the environment's emitted events is the expected event.
pub fn assert_emitted_event<U, V>(
    env: &Env,
    event_index: u32,
    contract_id: &Address,
    topics: U,
    data: V,
) where
    U: IntoVal<Env, Vec<Val>>,
    V: IntoVal<Env, Val>,
{
    let events = env.events().all();
    assert!(event_index < events.len(), "event_index out of bounds");

    let event = events.get(event_index).unwrap();

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(event.1, topics.into_val(env));
    assert_eq!(vec![env, event.2], vec![env, data.into_val(env)]);
}