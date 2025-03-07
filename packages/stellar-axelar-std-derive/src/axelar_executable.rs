use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

pub fn axelar_executable(name: &Ident) -> TokenStream2 {
    let error_alias = Ident::new(
        &format!("__{}AxelarExecutableInterfaceError", name),
        name.span(),
    );

    quote! {
        use stellar_axelar_gateway::executable::AxelarExecutableInterface as _;

        impl stellar_axelar_gateway::executable::DeriveOnly for #name {}

        #[allow(non_camel_case_types)]
        type #error_alias = <#name as stellar_axelar_gateway::executable::CustomAxelarExecutable>::Error;

        #[contractimpl]
        impl AxelarExecutableInterface for #name {
            fn gateway(env: &Env) -> Address {
                Self::__gateway(env)
            }

            fn execute(
                env: &Env,
                source_chain: String,
                message_id: String,
                source_address: String,
                payload: Bytes,
            ) -> Result<(), #error_alias> {
                stellar_axelar_gateway::executable::validate_message::<Self>(env, &source_chain, &message_id, &source_address, &payload).map_err(|err| match err {
                    stellar_axelar_gateway::executable::ValidationError::NotApproved => #error_alias::NotApproved,
                })?;

                Self::__execute(env, source_chain, message_id, source_address, payload)
            }
        }
    }
}
