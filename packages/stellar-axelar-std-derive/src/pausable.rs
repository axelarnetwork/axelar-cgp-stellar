use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::ItemFn;

use crate::modifier::modifier_impl;

pub fn pausable(name: &Ident) -> TokenStream2 {
    quote! {
        use stellar_axelar_std::interfaces::PausableInterface as _;

        #[soroban_sdk::contractimpl]
        impl stellar_axelar_std::interfaces::PausableInterface for #name {
            fn paused(env: &Env) -> bool {
                stellar_axelar_std::interfaces::paused(env)
            }

            fn pause(env: &Env) {
                stellar_axelar_std::interfaces::pause::<Self>(env);
            }

            fn unpause(env: &Env) {
                stellar_axelar_std::interfaces::unpause::<Self>(env);
            }
        }
    }
}

pub fn when_not_paused_impl(input_fn: ItemFn) -> TokenStream2 {
    modifier_impl(
        input_fn,
        quote! {
            stellar_axelar_std::ensure!(!Self::paused(env), ContractError::ContractPaused);
        },
    )
}
