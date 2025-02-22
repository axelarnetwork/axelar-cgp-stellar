//! Note: The tests are located in the `stellar-axelar-std` package instead of `stellar-axelar-std-derive`
//!
//! This ensures compatibility and prevents cyclic dependency issues during testing and release.

mod axelar_executable;
mod into_event;
mod its_executable;
mod modifier;
mod operatable;
mod ownable;
mod pausable;
mod storage;
mod upgradable;

use proc_macro::TokenStream;
use syn::{parse_macro_input, Attribute, DeriveInput, ItemFn, Path};

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

/// Implements the Pausable interface for a Soroban contract.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// # use soroban_sdk::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std_derive::Pausable;
///
/// #[contract]
/// #[derive(Pausable)]
/// pub struct Contract;
/// # }
/// ```
#[proc_macro_derive(Pausable)]
pub fn derive_pausable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    pausable::pausable(name).into()
}

/// Ensure that the Stellar contract is not paused before executing the function.
///
/// The first argument to the function must be `env`, and a `ContractError` error type must be defined in scope,
/// with a `ContractPaused` variant.
///
/// # Example
/// ```rust,ignore
/// # use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};
/// use stellar_axelar_std::{Pausable, when_not_paused};
///
/// #[contracttype]
/// pub enum ContractError {
///     ContractPaused = 1,
/// }
///
/// #[contract]
/// #[derive(Pausable)]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     #[when_not_paused]
///     pub fn transfer(env: &Env, to: Address, amount: String) {
///         // ... transfer logic ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn when_not_paused(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    pausable::when_not_paused_impl(input_fn).into()
}

/// Implements the Upgradable and Migratable interfaces for a Soroban contract.
///
/// A `ContractError` error type must be defined in scope, and have a `MigrationNotAllowed` variant.
/// A default migration implementation is automatically provided. If custom migration code is required,
/// the `#[migratable]` attribute can be applied to the contract struct.
/// In that case, the contract must implement the `CustomMigratableInterface` trait. The associated `Error` type
/// must implement the `Into<ContractError>` trait. The `ContractError` type itself implements it implicitly,
/// so that is an easy way to use it.
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
/// #[migratable]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(env: &Env, owner: Address) {
///         stellar_axelar_std::interfaces::set_owner(env, &owner);
///     }
/// }
///
/// impl CustomMigratableInterface for Contract {
///     type MigrationData = Address;
///     type Error = ContractError;
///
///     fn __migrate(env: &Env, new_owner: Self::MigrationData) -> Result<(), Self::Error> {
///         Self::transfer_ownership(env, new_owner);
///         Ok(())
///     }
/// }
/// # }
/// ```
#[proc_macro_derive(Upgradable, attributes(migratable))]
pub fn derive_upgradable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    upgradable::upgradable(&input).into()
}

fn ensure_no_args(attr: &Attribute) -> syn::Result<&Path> {
    attr.meta.require_path_only()
}

/// Implements the Event trait for a Stellar contract event.
///
/// Fields without a `#[data]` attribute are used as topics, while fields with `#[data]` are used as event data.
/// The event name can be specified with `#[event_name(...)]` or will default to the struct name in snake_case (minus "Event" suffix).
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// use core::fmt::Debug;
/// use stellar_axelar_std::events::Event;
/// use stellar_axelar_std::IntoEvent;
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

    into_event::into_event(&input).into()
}

#[proc_macro_derive(InterchainTokenExecutable)]
pub fn derive_its_executable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    its_executable::its_executable(name).into()
}

#[proc_macro_derive(AxelarExecutable)]
pub fn derive_axelar_executable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    axelar_executable::axelar_executable(name).into()
}

/// Ensures that only a contract's owner can execute the attributed function.
///
/// The first argument to the function must be `env`
///
/// # Example
/// ```rust,ignore
/// # use soroban_sdk::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std::only_owner;
///
/// #[contract]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     #[only_owner]
///     pub fn transfer(env: &Env, to: Address, amount: String) {
///         // ... transfer logic ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn only_owner(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    ownable::only_owner_impl(input_fn).into()
}

/// Ensures that only a contract's operator can execute the attributed function.
///
/// The first argument to the function must be `env`
///
/// # Example
/// ```rust,ignore
/// # use soroban_sdk::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std::only_operator;
///
/// #[contract]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     #[only_operator]
///     pub fn transfer(env: &Env, to: Address, amount: String) {
///         // ... transfer logic ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn only_operator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    operatable::only_operator_impl(input_fn).into()
}

/// Implements a storage interface for a Stellar contract storage enum.
///
/// The enum variants define contract data keys, with optional named fields as contract data map keys.
/// Each variant requires a `#[value(Type)]` xor `#[status]` attribute to specify the stored value type.
/// Storage type can be specified with `#[instance]`, `#[persistent]`, or `#[temporary]` attributes (defaults to instance).
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// use soroban_sdk::{contract, contractimpl, contractype, Address, Env, String};
/// use stellar_axelar_std::contractstorage;
///
/// #[contractstorage]
/// #[derive(Clone, Debug)]
/// enum DataKey {
///     #[instance]
///     #[value(Address)]
///     Owner,
///
///     #[persistent]
///     #[value(String)]
///     TokenName { token_id: u32 },
///
///     #[temporary]
///     #[value(u64)]
///     LastUpdate { account: Address },
///
///     #[instance]
///     #[status]
///     Paused,
/// }
///
/// #[contract]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(
///         env: &Env,
///         token_id: u32,
///         name: String,
///     ) {
///         storage::set_token_name(env, token_id, &name);
///     }
///
///     pub fn foo(env: &Env, token_id: u32) -> Option<String> {
///         storage::token_name(env, token_id);
///     }
///
///     pub fn bar(env: &Env, token_id: u32) -> Option<String> {
///         storage::remove_token_name(env, token_id)
///     }
/// }
/// # }
/// ```
#[proc_macro_attribute]
pub fn contractstorage(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    storage::contract_storage(&input).into()
}

trait MapTranspose<T> {
    fn map_transpose<U, E, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> Result<Option<U>, E>;
}

impl<T> MapTranspose<T> for Option<T> {
    fn map_transpose<U, E, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> Result<Option<U>, E> {
        self.map(f).transpose()
    }
}
