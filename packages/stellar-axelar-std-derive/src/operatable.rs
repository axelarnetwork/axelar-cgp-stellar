use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::ItemFn;

use crate::modifier::modifier_impl;

pub fn operatable(name: &Ident) -> TokenStream2 {
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
}
pub fn only_operator_impl(input_fn: ItemFn) -> TokenStream2 {
    modifier_impl(
        input_fn,
        quote! {
            Self::operator(&env).require_auth();
        },
    )
}
