/// Return with an error if a condition is not met.
///
/// Simplifies the pattern of checking for a condition and returning with an error.
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $e:expr $(,)?) => {
        if !$cond {
            return Err($e);
        }
    };
}

// The following definitions are mostly intended to serve as pseudo-documentation within tests
// and help with convenience/clarity.

/// Assert that a [`Result`] is [`Ok`]
///
/// If the provided expresion evaulates to [`Ok`], then the
/// macro returns the value contained within the [`Ok`]. If
/// the [`Result`] is an [`Err`] then the macro will [`panic`]
/// with a message that includes the expression and the error.
///
/// This function was vendored from [assert_ok](https://docs.rs/assert_ok/1.0.2/assert_ok/).
#[macro_export]
macro_rules! assert_ok {
    ( $x:expr ) => {
        match $x {
            Ok(v) => v,
            Err(e) => {
                panic!("Error calling {}: {:?}", stringify!($x), e);
            }
        }
    };
}

/// Assert that a [`Result`] is [`Err`] and matches an error variant
#[macro_export]
macro_rules! assert_err {
    ( $x:expr, $expected:pat ) => {
        match $x {
            Err(e) => {
                if !matches!(e, $expected) {
                    panic!("Expected error {}: {:?}", stringify!($expected), e);
                }
            }
            Ok(v) => {
                panic!("Expected error {}, found {:?}", stringify!($expected), v)
            }
        }
    };
}

/// Assert that a [`Result`] from a contract call is [`Err`] and matches an error variant
///
/// `given` corresponds to the return type from `try_*` functions in Soroban.
/// For the assert to succeed, the function needs to fail and successfully pass
/// down the intended error type. So, the parameters would be in the form:
///
/// given: `Err(Ok(ContractError))`
/// expected: `ContractError`
///
/// Putting it together in a function call:
///
/// `assert_contract_err(client.try_fun(...), ContractError);`
#[macro_export]
macro_rules! assert_contract_err {
    ($given:expr, $expected:expr) => {
        match $given {
            Ok(v) => panic!(
                "Expected error {:?}, got {:?} instead",
                stringify!($expected),
                v
            ),
            Err(e) => match e {
                Err(e) => panic!("Unexpected error {e:?}"),
                Ok(v) if v != $expected => {
                    panic!("Expected error {:?}, got {:?} instead", $expected, v)
                }
                _ => (),
            },
        }
    };
}

/// Assert that an [`Option`] is [`Some`]
///
/// If the provided expresion evaulates to [`Some`], then the
/// macro returns the value contained within the [`Some`]. If
/// the [`Option`] is [`None`] then the macro will [`panic`]
/// with a message that includes the expression
#[macro_export]
macro_rules! assert_some {
    ( $x:expr ) => {
        match $x {
            core::option::Option::Some(s) => s,
            core::option::Option::None => {
                panic!("Expected value when calling {}, got None", stringify!($x));
            }
        }
    };
}

/// Assert that a contract call with authentication succeeds
///
/// This macro is used to test contract calls that require authentication. It mocks the authentication
/// for the specified caller and executes the contract call. If the call fails or doesn't require the authentication,
/// the macro will panic with an error message.
///
/// # Example
///
/// ```rust,ignore
/// # use soroban_sdk::{Address, Env, contract, contractimpl};
/// # use soroban_sdk::testutils::Address as _;
/// # use stellar_axelar_std::assert_auth;
///
/// #[contract]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///    pub fn set_value(env: &Env, caller: Address, value: u32) {
///        caller.require_auth();
///    }
/// }
///
/// # let env = Env::default();
/// # let caller = Address::generate(&env);
/// # let contract_id = env.register(Contract, ());
/// # let client = ContractClient::new(&env, &contract_id);
///
/// assert_auth!(caller, client.set_value(&caller, &42));
/// ```
#[macro_export]
macro_rules! assert_auth {
    ($caller:expr, $client:ident . $method:ident ( $($arg:expr),* $(,)? )) => {{
        use soroban_sdk::IntoVal;

        // Evaluate the expression before the method call.
        // If the expression itself called the contract, e.g. client.owner(),
        // then this will prevent events from being reset when checking the auth after the call.
        let caller = $caller.clone();

        // Paste is used to concatenate the method name with the `try_` prefix
        paste::paste! {
        let result = $client
            .mock_auths(&[$crate::mock_auth!(
                caller,
                $client.$method($($arg),*)
            )])
            .[<try_ $method>]($($arg),*);
        }

        let result = match result {
            Ok(outer) => {
                match outer {
                    Ok(inner) => inner,
                    Err(err) => panic!("Expected Ok result, but got an error {:?}", err),
                }
            }
            Err(err) => panic!("Expected Ok result, but got an error {:?}", err),
        };

        assert_eq!(
            $client.env.auths(),
            std::vec![(
                caller,
                soroban_sdk::testutils::AuthorizedInvocation {
                    function: soroban_sdk::testutils::AuthorizedFunction::Contract((
                        $client.address.clone(),
                        soroban_sdk::Symbol::new(&$client.env, stringify!($method)),
                        ($($arg.clone(),)*).into_val(&$client.env)
                    )),
                    sub_invocations: std::vec![]
                }
            )]
        );

        result
    }};
}

#[macro_export]
macro_rules! assert_auth_err {
    ($caller:expr, $client:ident . $method:ident ( $($arg:expr),* $(,)? )) => {{
        use soroban_sdk::xdr::{ScError, ScErrorCode, ScVal};

        let caller = $caller.clone();

        paste::paste! {
        let call_result = $client
            .mock_auths(&[$crate::mock_auth!(
                caller,
                $client.$method($($arg),*)
            )])
            .[<try_ $method>]($($arg),*);
        }
        match call_result {
            Err(_) => {
                let val = ScVal::Error(ScError::Context(ScErrorCode::InvalidAction));
                match ScError::try_from(val) {
                    Ok(ScError::Context(ScErrorCode::InvalidAction)) => {}
                    _ => panic!("Expected ScErrorCode::InvalidAction"),
                }
            }
            Ok(_) => panic!("Expected error, but got Ok result."),
        }
    }};
}

#[macro_export]
macro_rules! mock_auth {
    (
        $caller:expr,
        $client:ident . $method:ident ( $($arg:expr),* $(,)? ),
        $sub_invokes:expr
    ) => {{
        use soroban_sdk::IntoVal;

        soroban_sdk::testutils::MockAuth {
            address: &$caller,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &$client.address,
                fn_name: &stringify!($method).replace("try_", ""),
                args: ($($arg.clone(),)*).into_val(&$client.env),
                sub_invokes: $sub_invokes,
            },
        }
    }};

    (
        $caller:expr,
        $client:ident . $method:ident ( $($arg:expr),* $(,)? )
    ) => {{
        $crate::mock_auth!($caller, $client.$method($($arg),*), &[])
    }};
}
