use itertools::Itertools;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::DeriveInput;

use crate::{ensure_no_args, MapTranspose};

pub fn upgradable(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    let migration_kind = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("migratable"))
        .at_most_one()
        .expect("migratable attribute can only be applied once")
        .map_transpose(ensure_no_args)
        .expect("migratable attribute cannot have arguments")
        .map(|_| MigrationKind::Custom)
        .unwrap_or_default();

    let custom_migration_impl = match migration_kind {
        MigrationKind::Default => default_custom_migration(name),
        MigrationKind::Custom => quote! {},
    };

    let migration_data_alias = Ident::new(&format!("__{}MigrationData", name), name.span());

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

        #[allow(non_camel_case_types)]
        type #migration_data_alias = <#name as stellar_axelar_std::interfaces::CustomMigratableInterface>::MigrationData;

        #[soroban_sdk::contractimpl]
        impl stellar_axelar_std::interfaces::MigratableInterface for #name {
            type Error = ContractError;

            fn migrate(env: &Env, migration_data: #migration_data_alias) -> Result<(), ContractError> {
                stellar_axelar_std::interfaces::migrate::<Self>(env, migration_data)
                    .map_err(|err| match err {
                        stellar_axelar_std::interfaces::MigrationError::NotAllowed => ContractError::MigrationNotAllowed,
                        stellar_axelar_std::interfaces::MigrationError::ExecutionFailed(err) => err.into(),
                    }
                )
            }
        }

        #custom_migration_impl
    }
}

fn default_custom_migration(name: &Ident) -> TokenStream2 {
    quote! {
        impl stellar_axelar_std::interfaces::CustomMigratableInterface for #name {
            type MigrationData = ();
            type Error = ContractError;

            fn __migrate(_env: &Env, _migration_data: Self::MigrationData) -> Result<(), Self::Error> {
                Ok(())
            }
        }
    }
}

#[derive(Default)]
pub enum MigrationKind {
    #[default]
    Default,
    Custom,
}

/// Tests the upgradable impl generation for a contract.
#[cfg(test)]
mod tests {

    #[test]
    fn upgradable_impl_generation_succeeds() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(Ownable, Upgradable)]
            #[migratable]
            pub struct Contract;
        };

        let upgradable_impl: proc_macro2::TokenStream =
            crate::upgradable::upgradable(&contract_input);
        let upgradable_impl_file: syn::File = syn::parse2(upgradable_impl).unwrap();
        let formatted_upgradable_impl = prettyplease::unparse(&upgradable_impl_file)
            .replace("pub fn ", "\npub fn ")
            .replace("#[cfg(test)]", "\n#[cfg(test)]");
        goldie::assert!(formatted_upgradable_impl);
    }
}
