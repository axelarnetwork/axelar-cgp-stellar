use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::ItemFn;

use crate::modifier::modifier_impl;

pub fn ownable(name: &Ident) -> TokenStream2 {
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
}

pub fn only_owner_impl(input_fn: ItemFn) -> TokenStream2 {
    modifier_impl(
        input_fn,
        quote! {
            Self::owner(&env).require_auth();
        },
    )
}
