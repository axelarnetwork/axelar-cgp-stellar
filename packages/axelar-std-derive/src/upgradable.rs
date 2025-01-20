use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Token, Type};

pub fn upgradable(name: &Ident, args: MigrationArgs) -> TokenStream2 {
    syn::parse_str::<Type>("ContractError").unwrap_or_else(|_| {
        panic!(
            "{}",
            Error::new(
                name.span(),
                "ContractError must be defined in scope.\n\
                 Hint: Add this to your code:\n\
                 #[contracterror]\n\
                 #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]\n\
                 #[repr(u32)]\n\
                 pub enum ContractError {\n    \
                     MigrationNotAllowed = 1,\n\
                     ...\n
                 }",
            )
            .to_string()
        )
    });

    let migration_data = args
        .migration_data
        .as_ref()
        .map_or_else(|| quote! { () }, |ty| quote! { #ty });

    quote! {
        use stellar_axelar_std::interfaces::{UpgradableInterface as _, MigratableInterface as _};

        #[soroban_sdk::contractimpl]
        impl stellar_axelar_std::interfaces::UpgradableInterface for #name {
            fn version(env: &Env) -> soroban_sdk::String {
                soroban_sdk::String::from_str(env, env!("CARGO_PKG_VERSION"))
            }

            fn upgrade(env: &Env, new_wasm_hash: soroban_sdk::BytesN<32>) {
                stellar_axelar_std::interfaces::upgrade::<Self>(env, new_wasm_hash);
            }
        }

        #[soroban_sdk::contractimpl]
        impl stellar_axelar_std::interfaces::MigratableInterface for #name {
            type MigrationData = #migration_data;
            type Error = ContractError;

            fn migrate(env: &Env, migration_data: #migration_data) -> Result<(), ContractError> {
                stellar_axelar_std::interfaces::migrate::<Self>(env, || Self::run_migration(env, migration_data))
                    .map_err(|_| ContractError::MigrationNotAllowed)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct MigrationArgs {
    migration_data: Option<Type>,
}

impl Parse for MigrationArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let migration_data = Some(Self::parse_migration_data(input)?);

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        Ok(Self { migration_data })
    }
}

impl MigrationArgs {
    fn parse_migration_data(input: ParseStream) -> syn::Result<Type> {
        let ident = input.parse::<Ident>()?;
        if ident != "with_type" {
            return Err(Error::new(ident.span(), "expected `with_type = ...`"));
        }

        input.parse::<Token![=]>()?;
        input.parse::<Type>()
    }
}
