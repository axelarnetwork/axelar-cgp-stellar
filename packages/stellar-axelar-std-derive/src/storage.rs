use std::convert::TryFrom;

use heck::ToSnakeCase;
use itertools::Itertools;
use prettyplease::unparse;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, FieldsNamed, Meta, Type, Variant};

enum Value {
    Status,
    Type(Type),
}

impl TryFrom<&[Attribute]> for Value {
    type Error = String;

    fn try_from(attrs: &[Attribute]) -> Result<Self, Self::Error> {
        let attr = attrs
            .iter()
            .filter(|attr| attr.path().is_ident("status") || attr.path().is_ident("value"))
            .exactly_one()
            .map_err(|_| {
                "exactly one of #[status] and #[value(Type)] must be provided".to_string()
            })?;

        if let Meta::List(list) = &attr.meta {
            if attr.path().is_ident("value") {
                Ok(Self::Type(
                    list.parse_args::<Type>()
                        .map_err(|_| "failed to parse value type".to_string())?,
                ))
            } else {
                Err("status attribute cannot have parameters".to_string())
            }
        } else if attr.path().is_ident("status") {
            Ok(Self::Status)
        } else {
            Err("value attribute must contain a type parameter: #[value(Type)]".to_string())
        }
    }
}

trait VariantExt {
    fn storage_params(&self) -> TokenStream;
    fn storage_key(&self, enum_name: &Ident) -> TokenStream;
}

impl VariantExt for Variant {
    /// Returns the parameter list for a storage enum variant.
    fn storage_params(&self) -> TokenStream {
        let (field_names, field_types) = fields_data(&self.fields);

        if field_names.is_empty() {
            quote! { env: &soroban_sdk::Env }
        } else {
            quote! { env: &soroban_sdk::Env, #(#field_names: #field_types),* }
        }
    }

    /// Returns the key for a storage enum variant.
    fn storage_key(&self, enum_name: &Ident) -> TokenStream {
        let (field_names, _) = fields_data(&self.fields);
        let variant_ident = &self.ident;

        if field_names.is_empty() {
            quote! { #enum_name::#variant_ident }
        } else {
            let field_names = field_names.iter().map(|name| quote! { #name });
            quote! { #enum_name::#variant_ident(#(#field_names),*) }
        }
    }
}

impl Value {
    /// Returns the getter, setter, and remover functions for a storage enum variant.
    fn fns(&self, enum_name: &Ident, storage_type: &StorageType, variant: &Variant) -> TokenStream {
        let (getter_name, setter_name, remover_name, try_getter_name) =
            self.fn_names(&variant.ident);
        let param_list = variant.storage_params();
        let storage_key = variant.storage_key(enum_name);

        match self {
            Self::Status => {
                let status_fns = storage_type.status_fns(
                    &getter_name,
                    &setter_name,
                    &remover_name,
                    &param_list,
                    &storage_key,
                );

                quote! { #status_fns }
            }
            Self::Type(value_type) => {
                let value_fns = storage_type.value_fns(
                    &getter_name,
                    &setter_name,
                    &remover_name,
                    &try_getter_name,
                    &param_list,
                    &storage_key,
                    value_type,
                );

                quote! { #value_fns }
            }
        }
    }

    /// Returns the getter, setter, and remover names for a storage enum variant.
    fn fn_names(&self, variant_ident: &Ident) -> (Ident, Ident, Ident, Ident) {
        let ident = variant_ident.to_string().to_snake_case();
        match self {
            Self::Status => (
                format_ident!("is_{}", ident),
                format_ident!("set_{}_status", ident),
                format_ident!("remove_{}_status", ident),
                format_ident!("_"),
            ),
            Self::Type(_) => (
                format_ident!("{}", ident),
                format_ident!("set_{}", ident),
                format_ident!("remove_{}", ident),
                format_ident!("try_{}", ident),
            ),
        }
    }
}

#[derive(Debug)]
enum StorageType {
    Instance,
    Persistent,
    Temporary,
}

impl TryFrom<&[Attribute]> for StorageType {
    type Error = String;

    fn try_from(attrs: &[Attribute]) -> Result<Self, Self::Error> {
        attrs
            .iter()
            .flat_map(|attr| match attr {
                _ if attr.path().is_ident("instance") => Some(Self::Instance),
                _ if attr.path().is_ident("persistent") => Some(Self::Persistent),
                _ if attr.path().is_ident("temporary") => Some(Self::Temporary),
                _ => None,
            })
            .exactly_one()
            .map_err(|_| {
                "storage type must be specified exactly once as 'instance', 'persistent', or 'temporary'"
                    .to_string()
            })
    }
}

impl StorageType {
    pub fn status_fns(
        &self,
        getter_name: &Ident,
        setter_name: &Ident,
        remover_name: &Ident,
        param_list: &TokenStream,
        key: &TokenStream,
    ) -> TokenStream {
        match self {
            Self::Persistent => quote! {
                pub fn #getter_name(#param_list) -> bool {
                    let value = env.storage()
                        .persistent()
                        .has(&#key);

                    if value {
                        stellar_axelar_std::ttl::extend_persistent_ttl(env, &#key)
                    }

                    value
                }

                pub fn #setter_name(#param_list) {
                    env.storage()
                        .persistent()
                        .set(&#key, &());
                }

                pub fn #remover_name(#param_list) {
                    env.storage()
                        .persistent()
                        .remove(&#key);
                }
            },
            Self::Instance => quote! {
                pub fn #getter_name(#param_list) -> bool {
                    let value = env.storage()
                        .instance()
                        .has(&#key);

                    if value {
                        stellar_axelar_std::ttl::extend_instance_ttl(env)
                    }

                    value
                }

                pub fn #setter_name(#param_list) {
                    env.storage()
                        .instance()
                        .set(&#key, &());
                }

                pub fn #remover_name(#param_list) {
                    env.storage()
                        .instance()
                        .remove(&#key);
                }
            },
            Self::Temporary => quote! {
                pub fn #getter_name(#param_list) -> bool {
                    env.storage()
                        .temporary()
                        .has(&#key)
                }

                pub fn #setter_name(#param_list) {
                    env.storage()
                        .temporary()
                        .set(&#key, &());
                }

                pub fn #remover_name(#param_list) {
                    env.storage()
                        .temporary()
                        .remove(&#key);
                }
            },
        }
    }

    pub fn value_fns(
        &self,
        getter_name: &Ident,
        setter_name: &Ident,
        remover_name: &Ident,
        try_getter_name: &Ident,
        param_list: &TokenStream,
        key: &TokenStream,
        value_type: &Type,
    ) -> TokenStream {
        match self {
            Self::Persistent => quote! {
                pub fn #getter_name(#param_list) -> #value_type {
                    let key = #key;
                    let value = env.storage()
                        .persistent()
                        .get::<_, #value_type>(&key)
                        .unwrap();

                    stellar_axelar_std::ttl::extend_persistent_ttl(env, &key);

                    value
                }

                pub fn #try_getter_name(#param_list) -> Option<#value_type> {
                    let key = #key;
                    let value = env.storage()
                        .persistent()
                        .get::<_, #value_type>(&key);

                    if value.is_some() {
                        stellar_axelar_std::ttl::extend_persistent_ttl(env, &key);
                    }

                    value
                }

                pub fn #setter_name(#param_list, value: &#value_type) {
                    let key = #key;
                    env.storage()
                        .persistent()
                        .set(&key, value);

                    stellar_axelar_std::ttl::extend_persistent_ttl(env, &key);
                }

                pub fn #remover_name(#param_list) {
                    env.storage()
                        .persistent()
                        .remove(&#key)
                }
            },
            Self::Instance => quote! {
                pub fn #getter_name(#param_list) -> #value_type {
                    let value = env.storage()
                        .instance()
                        .get::<_, #value_type>(&#key)
                        .unwrap();

                    stellar_axelar_std::ttl::extend_instance_ttl(env);

                    value
                }

                pub fn #try_getter_name(#param_list) -> Option<#value_type> {
                    let value = env.storage()
                        .instance()
                        .get::<_, #value_type>(&#key);

                    if value.is_some() {
                        stellar_axelar_std::ttl::extend_instance_ttl(env);
                    }

                    value
                }

                pub fn #setter_name(#param_list, value: &#value_type) {
                    env.storage()
                        .instance()
                        .set(&#key, value);

                    stellar_axelar_std::ttl::extend_instance_ttl(env);
                }

                pub fn #remover_name(#param_list) {
                    env.storage()
                        .instance()
                        .remove(&#key);
                }
            },
            Self::Temporary => quote! {
                pub fn #getter_name(#param_list) -> #value_type {
                    env.storage()
                        .temporary()
                        .get::<_, #value_type>(&#key)
                        .unwrap()
                }

                pub fn #try_getter_name(#param_list) -> Option<#value_type> {
                    env.storage()
                        .temporary()
                        .get::<_, #value_type>(&#key)
                }

                pub fn #setter_name(#param_list, value: &#value_type) {
                    env.storage()
                        .temporary()
                        .set(&#key, value);
                }

                pub fn #remover_name(#param_list) {
                    env.storage()
                        .temporary()
                        .remove(&#key);
                }
            },
        }
    }
}

/// Generates the storage enum and its associated functions.
pub fn contract_storage(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        panic!("contractstorage can only be used on enums");
    };

    let transformed_variants: Vec<_> = variants.iter().map(transform_variant).collect();

    let fns: Vec<_> = variants
        .iter()
        .map(|variant| {
            let storage_type = StorageType::try_from(variant.attrs.as_slice()).unwrap();
            let value = Value::try_from(variant.attrs.as_slice()).unwrap();

            value.fns(name, &storage_type, variant)
        })
        .collect();

    let contract_storage = quote! {
        #[contracttype]
        enum #name {
            #(#transformed_variants,)*
        }

        #(#fns)*
    };

    let contract_storage_tests = contract_storage_tests(name, input);

    quote! {
        #contract_storage

        #contract_storage_tests
    }
}

/// Transforms a contractstorage enum variant with named fields into a storage key map (tuple variant),
/// or a single storage key for a unit variant.
///
/// The Unit variant must be captured here to avoid suffixing non-map variants with "{}".
///
/// # Example
/// ```rust,ignore
/// /* Original */
/// #[contractstorage]
/// enum DataKey {
///     #[instance]
///     #[value(Address)]
///     Gateway, // Unit variant (would need `Gateway {},` otherwise)
///
///     #[temporary]
///     #[value(Address)]
///     Users { user: Address }, // Named variant (one or more fields)
/// }
///
/// /* Generated */
/// #[contracttype]
/// pub enum DataKey {
///     Gateway, // Unit variant (storage key)
///     Users(Address), // Tuple variant (storage key map)
/// }
/// ```
fn transform_variant(variant: &Variant) -> TokenStream {
    let variant_name = &variant.ident;

    match &variant.fields {
        Fields::Unit => {
            quote! {
                #variant_name
            }
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            let types = named.iter().map(|f| &f.ty);
            quote! {
                #variant_name(#(#types),*)
            }
        }
        _ => panic!("only unit variants or named fields are supported in storage enums"),
    }
}

/// Returns the field names and types of a storage enum variant.
fn fields_data(fields: &Fields) -> (Vec<&Option<Ident>>, Vec<&Type>) {
    match fields {
        Fields::Unit => (vec![], vec![]),
        Fields::Named(fields) => {
            let names = fields.named.iter().map(|f| &f.ident).collect();
            let types = fields.named.iter().map(|f| &f.ty).collect();
            (names, types)
        }
        _ => panic!("only unit variants or named fields are supported in storage enums"),
    }
}

/// Generates the storage schema tests for a storage enum.
fn contract_storage_tests(enum_name: &Ident, enum_input: &DeriveInput) -> TokenStream {
    let test_module_name = format_ident!(
        "{}_storage_layout_tests",
        enum_name.to_string().to_snake_case()
    );

    let test_name = format_ident!(
        "ensure_{}_storage_schema_is_unchanged",
        enum_name.to_string().to_snake_case()
    );

    let enum_file: syn::File = syn::parse2(quote! { #enum_input }).unwrap();
    let formatted_enum = unparse(&enum_file)
        .replace("    #[instance]", "\n    #[instance]")
        .replace("    #[persistent]", "\n    #[persistent]")
        .replace("    #[temporary]", "\n    #[temporary]");

    quote! {
        #[cfg(test)]
        mod #test_module_name {
            use goldie;

            #[test]
            fn #test_name() {
                goldie::assert!(#formatted_enum);
            }
        }
    }
}

/// Tests the storage schema generation for a storage enum.
#[cfg(test)]
mod tests {
    use syn::{DeriveInput, Fields};

    use super::fields_data;
    use crate::storage::contract_storage;

    #[test]
    fn storage_schema_generation_succeeds() {
        let enum_input: DeriveInput = syn::parse_quote! {
            enum DataKey {
                #[instance]
                #[value(u32)]
                Counter,

                #[persistent]
                #[value(String)]
                Message { sender: Address },

                #[temporary]
                #[value(Address)]
                LastCaller { timestamp: u64 },

                #[persistent]
                #[value(bool)]
                Flag { key: String, owner: Address },

                #[persistent]
                #[value(Option<String>)]
                OptionalMessage { id: u32 },

                #[instance]
                #[status]
                Initialized,

                #[persistent]
                #[status]
                Paused,
            }
        };

        let storage_module = contract_storage(&enum_input);
        let storage_module_file: syn::File = syn::parse2(storage_module).unwrap();
        let formatted_storage_module = prettyplease::unparse(&storage_module_file)
            .replace("pub fn ", "\npub fn ")
            .replace("#[cfg(test)]", "\n#[cfg(test)]");
        goldie::assert!(formatted_storage_module);
    }

    #[test]
    #[should_panic(expected = "contractstorage can only be used on enums")]
    fn non_enum_fails() {
        let input: DeriveInput = syn::parse_quote! {
            struct NotAnEnum {
                field: u32,
            }
        };

        contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "only unit variants or named fields are supported in storage enums")]
    fn tuple_variant_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value(u32)]
                TupleVariant(String, u32),
            }
        };

        contract_storage(&input);
    }

    #[test]
    #[should_panic(
        expected = "storage type must be specified exactly once as 'instance', 'persistent', or 'temporary'"
    )]
    fn missing_storage_type_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[value(u32)]
                Counter,
            }
        };

        contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "exactly one of #[status] and #[value(Type)] must be provided")]
    fn missing_value_and_status_attribute_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                Counter,
            }
        };

        contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "only unit variants or named fields are supported in storage enums")]
    fn fields_data_tuple_variant_fails() {
        let fields = Fields::Unnamed(syn::parse_quote! {
            (String, u32)
        });

        fields_data(&fields);
    }

    #[test]
    #[should_panic(expected = "exactly one of #[status] and #[value(Type)] must be provided")]
    fn status_and_value_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value(bool)]
                #[status]
                InvalidKey,
            }
        };

        contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "failed to parse value type")]
    fn invalid_value_type_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value(!@$Type)]
                InvalidKey,
            }
        };

        contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "status attribute cannot have parameters")]
    fn status_with_params_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[status(bool)]
                InvalidKey,
            }
        };

        contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "value attribute must contain a type parameter: #[value(Type)]")]
    fn value_without_type_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value]
                InvalidKey,
            }
        };

        contract_storage(&input);
    }
}
