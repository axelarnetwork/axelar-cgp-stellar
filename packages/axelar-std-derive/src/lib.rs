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

#[proc_macro_derive(Operatable)]
pub fn derive_operatable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    operatable::operatable(name).into()
}

#[proc_macro_derive(Ownable)]
pub fn derive_ownable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    ownable::ownable(name).into()
}

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
