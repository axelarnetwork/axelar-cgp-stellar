use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

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
