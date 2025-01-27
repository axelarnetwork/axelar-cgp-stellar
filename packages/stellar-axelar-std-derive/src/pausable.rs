use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::ItemFn;

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
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_name = &fn_sig.ident;
    let fn_generics = &fn_sig.generics;
    let fn_inputs = &fn_sig.inputs;
    let fn_output = &fn_sig.output;
    let fn_body = &input_fn.block;
    let fn_attrs = &input_fn.attrs;

    // Check that env is the first parameter
    let Some(syn::FnArg::Typed(pat_type)) = fn_inputs.first() else {
        panic!("First parameter must be a typed parameter")
    };
    let syn::Pat::Ident(pat_ident) = &*pat_type.pat else {
        panic!("First parameter must be a simple identifier")
    };
    assert!(
        pat_ident.ident == "env",
        "First parameter must be named 'env'"
    );

    // Ensure the function is not paused
    quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            stellar_axelar_std::ensure!(!Self::paused(env), ContractError::ContractPaused);

            #fn_body
        }
    }
}
