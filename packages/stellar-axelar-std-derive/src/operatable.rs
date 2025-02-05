use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::ItemFn;

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

    quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            Self::operator(&env).require_auth();

            #fn_body
        }
    }
}
