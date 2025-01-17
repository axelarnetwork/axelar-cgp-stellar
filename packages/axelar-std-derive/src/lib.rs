use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, Error, Ident, LitStr, Token, Type};

/// Implements the Operatable interface for a Soroban contract.
///
/// # Example
/// ```rust
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

    quote! {
        use stellar_axelar_std::interfaces::OperatableInterface as _;

        #[soroban_sdk::contractimpl]
        impl stellar_axelar_std::interfaces::OperatableInterface for #name {
            fn operator(env: &Env) -> soroban_sdk::Address {
                stellar_axelar_std::interfaces::operator(env)
            }

            fn transfer_operatorship(env: &Env, new_operator: soroban_sdk::Address) {
                stellar_axelar_std::interfaces::transfer_operatorship::<Self>(env, new_operator);
            }
        }
    }
    .into()
}

/// Implements the Ownable interface for a Soroban contract.
///
/// # Example
/// ```rust
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

    quote! {
        use stellar_axelar_std::interfaces::OwnableInterface as _;

        #[soroban_sdk::contractimpl]
        impl stellar_axelar_std::interfaces::OwnableInterface for #name {
            fn owner(env: &Env) -> soroban_sdk::Address {
                stellar_axelar_std::interfaces::owner(env)
            }

            fn transfer_ownership(env: &Env, new_owner: soroban_sdk::Address) {
                stellar_axelar_std::interfaces::transfer_ownership::<Self>(env, new_owner);
            }
        }
    }
    .into()
}

#[derive(Debug, Default)]
struct MigrationArgs {
    migration_data: Option<Type>,
}

impl Parse for MigrationArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let migration_data = Some(Self::parse_migration_data(input)?);

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        Ok(Self { migration_data })
    }
}

impl MigrationArgs {
    fn parse_migration_data(input: ParseStream) -> syn::Result<Type> {
        let ident = input.parse::<Ident>()?;
        if ident != "with_type" {
            return Err(Error::new(ident.span(), "expected `with_type = ...`"));
        }

        input.parse::<Token![=]>()?;
        input.parse::<Type>()
    }
}

/// Implements the Upgradable and Migratable interfaces for a Soroban contract.
///
/// A `ContractError` error type must be defined in scope, and have a `MigrationNotAllowed` variant.
///
/// # Example
/// ```rust
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

    syn::parse_str::<Type>("ContractError").unwrap_or_else(|_| {
        panic!(
            "{}",
            Error::new(
                name.span(),
                "ContractError must be defined in scope.\n\
                 Hint: Add this to your code:\n\
                 #[contracterror]\n\
                 #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]\n\
                 #[repr(u32)]\n\
                 pub enum ContractError {\n    \
                     MigrationNotAllowed = 1,\n\
                     ...\n
                 }",
            )
            .to_string()
        )
    });

    let migration_data = args
        .migration_data
        .as_ref()
        .map_or_else(|| quote! { () }, |ty| quote! { #ty });

    quote! {
        use stellar_axelar_std::interfaces::{UpgradableInterface as _, MigratableInterface as _};

        #[soroban_sdk::contractimpl]
        impl stellar_axelar_std::interfaces::UpgradableInterface for #name {
            fn version(env: &Env) -> soroban_sdk::String {
                soroban_sdk::String::from_str(env, env!("CARGO_PKG_VERSION"))
            }

            fn upgrade(env: &Env, new_wasm_hash: soroban_sdk::BytesN<32>) {
                stellar_axelar_std::interfaces::upgrade::<Self>(env, new_wasm_hash);
            }
        }

        #[soroban_sdk::contractimpl]
        impl stellar_axelar_std::interfaces::MigratableInterface for #name {
            type MigrationData = #migration_data;
            type Error = ContractError;

            fn migrate(env: &Env, migration_data: #migration_data) -> Result<(), ContractError> {
                stellar_axelar_std::interfaces::migrate::<Self>(env, || Self::run_migration(env, migration_data))
                    .map_err(|_| ContractError::MigrationNotAllowed)
            }
        }
    }.into()
}

/// Implements the Event and EventTestUtils traits for a Soroban contract event.
///
/// Fields without a `#[data]` attribute are used as topics, while fields with `#[data]` are used as event data.
/// The event name can be specified with `#[event_name(...)]` or will default to the struct name in snake_case (minus "Event" suffix).
///
/// # Example
/// ```rust
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

    let event_impl = derive_event_impl(&input);

    #[cfg(any(test, feature = "testutils"))]
    let event_impl = {
        let event_test_impl = derive_event_testutils_impl(&input);
        quote! {
            #event_impl
            #event_test_impl
        }
    };

    event_impl.into()
}

fn derive_event_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let event_name = event_name_snake_case(input);
    let ((topic_idents, _), (data_idents, _)) = event_struct_fields(input);

    quote! {
        impl stellar_axelar_std::events::Event for #name {
            fn topics(&self, env: &soroban_sdk::Env) -> impl soroban_sdk::Topics + core::fmt::Debug {
                (
                    soroban_sdk::Symbol::new(env, #event_name),
                    #(soroban_sdk::IntoVal::<soroban_sdk::Env, soroban_sdk::Val>::into_val(&self.#topic_idents, env),)*
                )
            }

            fn data(&self, env: &soroban_sdk::Env) -> impl soroban_sdk::IntoVal<soroban_sdk::Env, soroban_sdk::Val> + core::fmt::Debug {
                let vec: soroban_sdk::Vec<soroban_sdk::Val> = soroban_sdk::vec![env, #(soroban_sdk::IntoVal::<_, soroban_sdk::Val>::into_val(&self.#data_idents, env))*];
                vec
            }

            fn emit(self, env: &soroban_sdk::Env) {
                env.events().publish(self.topics(env), self.data(env));
            }
        }
    }
}
#[cfg(any(test, feature = "testutils"))]
fn derive_event_testutils_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let ((_, topic_types), (_, data_types)) = event_struct_fields(input);
    event_testutils(name, topic_types, data_types)
}

#[cfg(any(test, feature = "testutils"))]
fn event_testutils(
    name: &Ident,
    topic_types: Vec<&Type>,
    data_types: Vec<&Type>,
) -> proc_macro2::TokenStream {
    quote! {
        impl stellar_axelar_std::events::EventTestutils for #name {
            fn matches(self, env: &soroban_sdk::Env, event: &(soroban_sdk::Address, soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val)) -> bool {
                use soroban_sdk::IntoVal;
                Self::standardized_fmt(env, event) == Self::standardized_fmt(env, &(event.0.clone(), self.topics(env).into_val(env), self.data(env).into_val(env)))
            }

            #[allow(unused_assignments)]
            #[allow(unused_variables)]
            #[allow(unused_mut)]
            fn standardized_fmt(env: &soroban_sdk::Env, (contract_id, topics, data): &(soroban_sdk::Address, soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val)) -> std::string::String {
                use soroban_sdk::TryFromVal;

                let mut topics_output: std::vec::Vec<std::string::String> = std::vec![];

                let event_name = topics.get(0).expect("event name topic missing");
                topics_output.push(std::format!("{:?}", soroban_sdk::Symbol::try_from_val(env, &event_name)
                    .expect("event name should be a Symbol")));

                let mut i = 1;
                #(
                    let topic = topics.get(i).expect("the number of topics does not match this function's definition");
                    topics_output.push(std::format!("{:?}", <#topic_types>::try_from_val(env, &topic)
                        .expect("given topic value does not match the expected type")));

                    i += 1;
                )*

                let data = soroban_sdk::Vec::<soroban_sdk::Val>::try_from_val(env, data)
                    .expect("data should be defined as a vector-compatible type");

                let mut data_output: std::vec::Vec<std::string::String> = std::vec![];

                let mut i = 0;
                #(
                    let data_entry = data.get(i).expect("the number of data entries does not match this function's definition");
                    data_output.push(std::format!("{:?}", <#data_types>::try_from_val(env, &data_entry)
                        .expect("given data value does not match the expected type")));

                    i += 1;
                )*

                std::format!("contract: {:?}\ntopics: ({})\ndata: ({})",
                    contract_id,
                    topics_output.join(", "),
                    data_output.join(", ")
                )
            }
        }
    }
}

fn event_name_snake_case(input: &DeriveInput) -> String {
    input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("event_name"))
        .map(|attr| attr.parse_args::<LitStr>().unwrap().value())
        .unwrap_or_else(|| {
            input
                .ident
                .to_string()
                .strip_suffix("Event")
                .unwrap()
                .to_snake_case()
        })
}

type EventIdent<'a> = Vec<&'a Ident>;
type EventType<'a> = Vec<&'a Type>;
type EventStructFields<'a> = (EventIdent<'a>, EventType<'a>);

fn event_struct_fields(input: &DeriveInput) -> (EventStructFields, EventStructFields) {
    let syn::Data::Struct(data_struct) = &input.data else {
        panic!("IntoEvent can only be derived for structs");
    };

    let mut topic_idents = Vec::new();
    let mut topic_types = Vec::new();
    let mut data_idents = Vec::new();
    let mut data_types = Vec::new();

    for field in data_struct.fields.iter() {
        if let Some(ident) = field.ident.as_ref() {
            if field.attrs.iter().any(|attr| attr.path().is_ident("data")) {
                data_idents.push(ident);
                data_types.push(&field.ty);
            } else {
                topic_idents.push(ident);
                topic_types.push(&field.ty);
            }
        }
    }

    ((topic_idents, topic_types), (data_idents, data_types))
}
