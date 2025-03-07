#![cfg(any(test, feature = "testutils"))]
extern crate std;

/// Helper macro for building and verifying authorization chains in Soroban contract tests.
///
/// Used to verify that contract calls require the correct sequence of authorizations.
/// See the example package for usage in gas payment and cross-chain message verification scenarios.
///
/// # Example
/// ```rust,ignore
/// // Create authorization
/// let transfer_auth = auth_invocation!(
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
    ($caller:expr, $client:ident.$method:ident($($arg:expr),* $(,)?)) => {{
        std::vec![(
            $caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    $client.address.clone(),
                    Symbol::new(&$client.env, stringify!($method)),
                    ($($arg),*).into_val(&$client.env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    }};

    // Case with sub-invocations (handles both regular and user auth cases)
    ($caller:expr, $client:ident.$method:ident($($arg:expr),* $(,)?), $subs:expr $(, $user:ident)?) => {{
        std::vec![(
            $caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    $client.address.clone(),
                    Symbol::new(&$client.env, stringify!($method)),
                    ($($arg),*).into_val(&$client.env),
                )),
                sub_invocations: $subs.into_iter().map(|(_, inv)| inv).collect(),
            }
        )]
    }};
}
