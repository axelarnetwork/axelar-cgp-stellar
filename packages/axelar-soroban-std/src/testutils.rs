#![cfg(any(test, feature = "testutils"))]
extern crate std;

use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation, Events};
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol, Val, Vec};

/// Asserts invocation auth of a contract from a single caller.
pub fn assert_invocation<T>(
    env: &Env,
    caller: &Address,
    contract_id: &Address,
    function_name: &str,
    args: T,
) where
    T: IntoVal<Env, Vec<Val>>,
{
    assert_eq!(
        env.auths(),
        std::vec![(
            caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(env, function_name),
                    args.into_val(env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

/// Asserts that the event at `event_index` in the environment's emitted events is the expected event.
/// If `event_index` is negative, the length of events will be added to it, i.e it'll be indexed from the end.
pub fn assert_emitted_event<U, V>(
    env: &Env,
    mut event_index: i32,
    contract_id: &Address,
    topics: U,
    data: V,
) where
    U: IntoVal<Env, Vec<Val>>,
    V: IntoVal<Env, Val>,
{
    let events = env.events().all();
    if event_index.is_negative() {
        event_index += events.len() as i32;
    }

    assert!(
        event_index < events.len() as i32,
        "event {} not found, only {} events were emitted",
        event_index + 1,
        events.len()
    );

    let event = events.get(event_index as u32).unwrap();

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(event.1, topics.into_val(env));
    assert_eq!(vec![env, event.2], vec![env, data.into_val(env)]);
}

pub fn assert_last_emitted_event<U, V>(env: &Env, contract_id: &Address, topics: U, data: V)
where
    U: IntoVal<Env, Vec<Val>>,
    V: IntoVal<Env, Val>,
{
    assert_emitted_event(env, -1, contract_id, topics, data);
}

/// Helper macro for building and verifying authorization chains in Soroban contract tests.
///
/// Used to verify that contract calls require the correct sequence of authorizations.
/// See the example package for usage in gas payment and cross-chain message verification scenarios.
///
/// # Example
/// ```rust,ignore
/// // Create authorization for a token transfer
/// let transfer_auth = auth_invocation!(
///     &env,
///     user,
///     asset_client.transfer(
///         &user,
///         source_gas_service_id,
///         gas_token.amount
///     )
/// );
///
/// // Create nested authorization chain for gas payment
/// let pay_gas_auth = auth_invocation!(
///     &env,
///     user,
///     source_gas_service_client.pay_gas(
///         source_app.address,
///         destination_chain,
///         destination_address,
///         payload,
///         &user,
///         gas_token,
///         &Bytes::new(&env)
///     ),
///     transfer_auth
/// );
///
/// // Verify authorizations
/// assert_eq!(env.auths(), pay_gas_auth);
/// ```
#[macro_export]
macro_rules! auth_invocation {
    // Basic case without sub-invocations
    ($env:expr, $caller:expr, $client:ident.$method:ident($($arg:expr),* $(,)?)) => {{
        std::vec![(
            $caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    $client.address.clone(),
                    Symbol::new($env, stringify!($method)),
                    ($($arg),*).into_val($env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    }};

    // Case with sub-invocations (handles both regular and user auth cases)
    ($env:expr, $caller:expr, $client:ident.$method:ident($($arg:expr),* $(,)?), $subs:expr $(, $user:ident)?) => {{
        std::vec![(
            $caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    $client.address.clone(),
                    Symbol::new($env, stringify!($method)),
                    ($($arg),*).into_val($env),
                )),
                sub_invocations: $subs.into_iter().map(|(_, inv)| inv).collect(),
            }
        )]
    }};
}
