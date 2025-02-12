use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ItemFn;

pub fn modifier_impl(input_fn: ItemFn, auth_statement: TokenStream2) -> TokenStream2 {
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_name = &fn_sig.ident;
    let fn_generics = &fn_sig.generics;
    let fn_inputs = &fn_sig.inputs;
    let fn_output = &fn_sig.output;
    let fn_body = &input_fn.block;
    let fn_attrs = &input_fn.attrs;

    let Some(syn::FnArg::Typed(pat_type)) = fn_inputs.first() else {
        panic!("first parameter must be a typed parameter")
    };
    let syn::Pat::Ident(pat_ident) = &*pat_type.pat else {
        panic!("first parameter must be a simple identifier")
    };
    assert!(
        pat_ident.ident == "env",
        "first parameter must be named 'env'"
    );

    quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            #auth_statement

            #fn_body
        }
    }
}

#[cfg(test)]
mod tests {

    use syn::{parse_quote, ItemFn};

    use super::*;

    #[test]
    fn first_parameter_is_typed() {
        let input_fn: ItemFn = parse_quote! {
            fn test_fn(env: &Env, other: i32) {}
        };
        let generated_function = modifier_impl(
            input_fn,
            quote! {
                Self::operator(&env).require_auth();
            },
        )
        .to_string();
        let generated_function_file: syn::File = syn::parse_str(&generated_function).unwrap();
        let formatted_generated_function = prettyplease::unparse(&generated_function_file)
            .replace("{}", "")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        goldie::assert!(formatted_generated_function);
    }

    #[test]
    #[should_panic(expected = "first parameter must be a typed parameter")]
    fn first_parameter_is_not_typed() {
        let input_fn: ItemFn = parse_quote! {
            fn test_fn() {}
        };
        modifier_impl(
            input_fn,
            quote! {
                Self::operator(&env).require_auth();
            },
        );
    }

    #[test]
    #[should_panic(expected = "first parameter must be a simple identifier")]
    fn first_parameter_is_not_simple_identifier() {
        let input_fn: ItemFn = parse_quote! {
            fn test_fn((env, other): (&Env, i32)) {}
        };
        modifier_impl(
            input_fn,
            quote! {
                Self::operator(&env).require_auth();
            },
        );
    }

    #[test]
    #[should_panic(expected = "first parameter must be named 'env'")]
    fn first_parameter_is_not_named_env() {
        let input_fn: ItemFn = parse_quote! {
            fn test_fn(not_env: &Env, other: i32) {}
        };
        modifier_impl(
            input_fn,
            quote! {
                Self::operator(&env).require_auth();
            },
        );
    }
}
