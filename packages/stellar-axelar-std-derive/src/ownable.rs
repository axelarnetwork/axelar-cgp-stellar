use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::ItemFn;

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
            Self::owner(&env).require_auth();

            #fn_body
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, ItemFn};

    use super::*;

    #[test]
    fn test_first_parameter_is_typed() {
        let input_fn: ItemFn = parse_quote! {
            fn test_fn(env: &Env, other: i32) {}
        };
        only_owner_impl(input_fn);
    }

    #[test]
    #[should_panic(expected = "First parameter must be a typed parameter")]
    fn test_first_parameter_is_not_typed() {
        let input_fn: ItemFn = parse_quote! {
            fn test_fn() {}
        };
        only_owner_impl(input_fn);
    }

    #[test]
    fn test_first_parameter_is_simple_identifier() {
        let input_fn: ItemFn = parse_quote! {
            fn test_fn(env: &Env, other: i32) {}
        };
        only_owner_impl(input_fn);
    }

    #[test]
    #[should_panic(expected = "First parameter must be a simple identifier")]
    fn test_first_parameter_is_not_simple_identifier() {
        let input_fn: ItemFn = parse_quote! {
            fn test_fn((env, other): (&Env, i32)) {}
        };
        only_owner_impl(input_fn);
    }

    #[test]
    #[should_panic(expected = "First parameter must be named 'env'")]
    fn test_first_parameter_is_not_named_env() {
        let input_fn: ItemFn = parse_quote! {
            fn test_fn(not_env: &Env, other: i32) {}
        };
        only_owner_impl(input_fn);
    }
}
