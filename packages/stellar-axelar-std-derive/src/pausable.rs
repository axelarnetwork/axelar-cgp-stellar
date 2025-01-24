use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

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
