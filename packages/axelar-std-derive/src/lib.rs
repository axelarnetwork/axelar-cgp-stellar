//! Note: The tests are located in the `stellar-axelar-std` package instead of `stellar-axelar-std-derive`
//!
//! This ensures compatibility and prevents cyclic dependency issues during testing and release.

mod event;
mod its_executable;
mod operatable;
mod ownable;
mod upgradable;

use proc_macro::TokenStream;
#[cfg(any(test, feature = "testutils"))]
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use upgradable::MigrationArgs;

/// Implements the Operatable interface for a Soroban contract.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// # use soroban_sdk::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std_derive::Operatable;
///
/// #[contract]
/// #[derive(Operatable)]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(env: &Env, owner: Address) {
///         stellar_axelar_std::interfaces::set_operator(env, &owner);
///     }
/// }
/// # }
/// ```
#[proc_macro_derive(Operatable)]
pub fn derive_operatable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    operatable::operatable(name).into()
}

/// Implements the Ownable interface for a Soroban contract.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// # use soroban_sdk::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std_derive::Ownable;
///
/// #[contract]
/// #[derive(Ownable)]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(env: &Env, owner: Address) {
///         stellar_axelar_std::interfaces::set_owner(env, &owner);
///     }
/// }
/// # }
/// ```
#[proc_macro_derive(Ownable)]
pub fn derive_ownable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    ownable::ownable(name).into()
}

/// Implements the Upgradable and Migratable interfaces for a Soroban contract.
///
/// A `ContractError` error type must be defined in scope, and have a `MigrationNotAllowed` variant.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// # use soroban_sdk::{contract, contractimpl, contracterror, Address, Env};
/// use stellar_axelar_std_derive::{Ownable, Upgradable};
/// # #[contracterror]
/// # #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
/// # #[repr(u32)]
/// # pub enum ContractError {
/// #     MigrationNotAllowed = 1,
/// # }
///
/// #[contract]
/// #[derive(Ownable, Upgradable)]
/// #[migratable(with_type = Address)]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(env: &Env, owner: Address) {
///         stellar_axelar_std::interfaces::set_owner(env, &owner);
///     }
/// }
///
/// impl Contract {
///     fn run_migration(env: &Env, new_owner: Address) {
///         Self::transfer_ownership(env, new_owner);
///     }
/// }
/// # }
/// ```
#[proc_macro_derive(Upgradable, attributes(migratable))]
pub fn derive_upgradable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let args = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("migratable"))
        .map(|attr| attr.parse_args::<MigrationArgs>())
        .transpose()
        .unwrap_or_else(|e| panic!("{}", e))
        .unwrap_or_else(MigrationArgs::default);

    upgradable::upgradable(name, args).into()
}

/// Implements the Event and EventTestUtils traits for a Soroban contract event.
///
/// Fields without a `#[data]` attribute are used as topics, while fields with `#[data]` are used as event data.
/// The event name can be specified with `#[event_name(...)]` or will default to the struct name in snake_case (minus "Event" suffix).
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// use core::fmt::Debug;
/// use stellar_axelar_std::events::Event;
/// use stellar_axelar_std_derive::IntoEvent;
/// use soroban_sdk::{Address, contract, contractimpl, Env, String};
///
/// #[derive(Debug, PartialEq, IntoEvent)]
/// #[event_name("transfer")]
/// pub struct TransferEvent {
///     pub from: Address,
///     pub to: Address,
///     #[data]
///     pub amount: String,
/// }
///
/// #[contract]
/// pub struct Token;
///
/// #[contractimpl]
/// impl Token {
///     pub fn transfer(env: &Env, to: Address, amount: String) {
///         // ... transfer logic ...
///
///         // Generates event with:
///         // - Topics: ["transfer", contract_address, to]
///         // - Data: [amount]
///         TransferEvent {
///             from: env.current_contract_address(),
///             to,
///             amount,
///         }.emit(env);
///     }
/// }
/// }
/// ```
#[proc_macro_derive(IntoEvent, attributes(data, event_name))]
pub fn derive_into_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let event_impl = event::derive_event_impl(&input);

    #[cfg(any(test, feature = "testutils"))]
    let event_impl = {
        let event_test_impl = event::derive_event_testutils_impl(&input);
        quote! {
            #event_impl
            #event_test_impl
        }
    };

    event_impl.into()
}

#[proc_macro_derive(InterchainTokenExecutable)]
pub fn derive_its_executable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    its_executable::its_executable(name).into()
}
