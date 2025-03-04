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
            .map_err(|_| "exactly one of #[status] and #[value(Type)] must be provided")?;

        if attr.path().is_ident("status") {
            Ok(Self::Status)
        } else if let Meta::List(list) = &attr.meta {
            Ok(Self::Type(
                list.parse_args::<Type>()
                    .map_err(|_| "failed to parse value type")?,
            ))
        } else {
            Err("value attribute must contain a type parameter: #[value(Type)]".into())
        }
    }
}

trait FieldsExt {
    fn names(&self) -> Vec<&Ident>;

    fn types(&self) -> Vec<&Type>;
}

impl FieldsExt for Fields {
    /// Returns the field names of a storage enum variant.
    fn names(&self) -> Vec<&Ident> {
        match self {
            Self::Unit => vec![],
            Self::Named(fields) => fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect(),
            _ => panic!("only unit variants or named fields are supported in storage enums"),
        }
    }

    /// Returns the field types of a storage enum variant.
    fn types(&self) -> Vec<&Type> {
        match self {
            Self::Unit => vec![],
            Self::Named(fields) => fields.named.iter().map(|f| &f.ty).collect(),
            _ => panic!("only unit variants or named fields are supported in storage enums"),
        }
    }
}

trait VariantExt {
    fn storage_params(&self) -> TokenStream;
    fn storage_key(&self, r#enum: &Ident) -> TokenStream;
}

impl VariantExt for Variant {
    /// Returns the parameter list for a storage enum variant.
    fn storage_params(&self) -> TokenStream {
        let (field_names, field_types) = (self.fields.names(), self.fields.types());

        if field_names.is_empty() {
            quote! { env: &soroban_sdk::Env }
        } else {
            quote! { env: &soroban_sdk::Env, #(#field_names: #field_types),* }
        }
    }

    /// Returns the key for a storage enum variant.
    fn storage_key(&self, r#enum: &Ident) -> TokenStream {
        let field_names = self.fields.names();
        let variant_ident = &self.ident;

        if field_names.is_empty() {
            quote! { #r#enum::#variant_ident }
        } else {
            let field_names = field_names.iter().map(|name| quote! { #name });
            quote! { #r#enum::#variant_ident(#(#field_names),*) }
        }
    }
}

struct StorageFunctionNames {
    getter: Ident,
    setter: Ident,
    remover: Ident,
    try_getter: Ident,
    ttl_extender: Ident,
}

impl Value {
    /// Returns the getter, setter, and remover functions for a storage enum variant.
    fn storage_functions(
        &self,
        r#enum: &Ident,
        storage_type: &StorageType,
        variant: &Variant,
    ) -> TokenStream {
        let storage_function_names = self.storage_function_names(&variant.ident);
        let params = variant.storage_params();
        let storage_key = variant.storage_key(r#enum);

        match self {
            Self::Status => {
                let status_functions = self.status_functions(
                    storage_type,
                    &storage_function_names,
                    &params,
                    &storage_key,
                );

                quote! { #status_functions }
            }
            Self::Type(value_type) => {
                let value_functions = self.value_functions(
                    storage_type,
                    &storage_function_names,
                    &params,
                    &storage_key,
                    value_type,
                );

                quote! { #value_functions }
            }
        }
    }

    fn status_functions(
        &self,
        storage_type: &StorageType,
        StorageFunctionNames {
            getter,
            setter,
            remover,
            ttl_extender,
            ..
        }: &StorageFunctionNames,
        params: &TokenStream,
        storage_key: &TokenStream,
    ) -> TokenStream {
        let storage_method = storage_type.storage_method();
        let ttl_function = storage_type.ttl_function(&quote! { key });
        let default_ttl_function = storage_type.default_ttl_function(&quote! { key });

        quote! {
            pub fn #getter(#params) -> bool {
                let key = #storage_key;
                let value = #storage_method.has(&key);

                if value {
                    #default_ttl_function
                }

                value
            }

            pub fn #setter(#params) {
                let key = #storage_key;
                #storage_method.set(&key, &());

                #default_ttl_function
            }

            pub fn #remover(#params) {
                let key = #storage_key;
                #storage_method.remove(&key);
            }

            pub fn #ttl_extender(#params, threshold: u32, extend_to: u32) {
                let key = #storage_key;
                #ttl_function
            }
        }
    }

    fn value_functions(
        &self,
        storage_type: &StorageType,
        StorageFunctionNames {
            getter,
            setter,
            remover,
            try_getter,
            ttl_extender,
        }: &StorageFunctionNames,
        params: &TokenStream,
        storage_key: &TokenStream,
        value_type: &Type,
    ) -> TokenStream {
        let storage_method = storage_type.storage_method();
        let ttl_function = storage_type.ttl_function(&quote! { key });
        let default_ttl_function = storage_type.default_ttl_function(&quote! { key });

        quote! {
            pub fn #getter(#params) -> #value_type {
                let key = #storage_key;
                let value = #storage_method
                    .get::<_, #value_type>(&key)
                    .unwrap();

                #default_ttl_function

                value
            }

            pub fn #try_getter(#params) -> Option<#value_type> {
                let key = #storage_key;
                let value = #storage_method.get::<_, #value_type>(&key);

                if value.is_some() {
                    #default_ttl_function
                }

                value
            }

            pub fn #setter(#params, value: &#value_type) {
                let key = #storage_key;
                #storage_method.set(&key, value);

                #default_ttl_function
            }

            pub fn #remover(#params) {
                let key = #storage_key;
                #storage_method.remove(&key);
            }

            pub fn #ttl_extender(#params, threshold: u32, extend_to: u32) {
                let key = #storage_key;
                #ttl_function
            }
        }
    }

    /// Returns the getter, setter, and remover names for a storage enum variant.
    fn storage_function_names(&self, variant_ident: &Ident) -> StorageFunctionNames {
        let ident = variant_ident.to_string().to_snake_case();
        match self {
            Self::Status => StorageFunctionNames {
                getter: format_ident!("is_{}", ident),
                setter: format_ident!("set_{}_status", ident),
                remover: format_ident!("remove_{}_status", ident),
                try_getter: format_ident!("_"),
                ttl_extender: format_ident!("extend_{}_ttl", ident),
            },
            Self::Type(_) => StorageFunctionNames {
                getter: format_ident!("{}", ident),
                setter: format_ident!("set_{}", ident),
                remover: format_ident!("remove_{}", ident),
                try_getter: format_ident!("try_{}", ident),
                ttl_extender: format_ident!("extend_{}_ttl", ident),
            },
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
    fn storage_method(&self) -> TokenStream {
        match self {
            Self::Persistent => quote! { env.storage().persistent() },
            Self::Instance => quote! { env.storage().instance() },
            Self::Temporary => quote! { env.storage().temporary() },
        }
    }

    fn ttl_function(&self, key: &TokenStream) -> TokenStream {
        match self {
            Self::Persistent => {
                quote! { env.storage().persistent().extend_ttl(&#key, threshold, extend_to); }
            }
            Self::Instance => quote! { env.storage().instance().extend_ttl(threshold, extend_to); },
            Self::Temporary => {
                quote! { env.storage().temporary().extend_ttl(&#key, threshold, extend_to); }
            }
        }
    }

    fn default_ttl_function(&self, key: &TokenStream) -> TokenStream {
        match self {
            Self::Persistent => {
                quote! { stellar_axelar_std::ttl::extend_persistent_ttl(env, &#key); }
            }
            Self::Instance => quote! { stellar_axelar_std::ttl::extend_instance_ttl(env); },
            Self::Temporary => quote! {},
        }
    }
}

/// Generates the storage enum and its associated functions.
pub fn contract_storage(input: &DeriveInput) -> TokenStream {
    let r#enum = &input.ident;

    if matches!(input.vis, syn::Visibility::Public(_)) {
        panic!("contractstorage can only be used on private enums (remove 'pub' keyword)");
    }

    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        panic!("contractstorage can only be used on enums");
    };

    let transformed_variants: Vec<_> = variants.iter().map(transform_variant).collect();

    let storage_functions: Vec<_> = variants
        .iter()
        .map(|variant| {
            let storage_type = StorageType::try_from(variant.attrs.as_slice()).unwrap();
            let value = Value::try_from(variant.attrs.as_slice()).unwrap();

            value.storage_functions(r#enum, &storage_type, variant)
        })
        .collect();

    let contract_storage = quote! {
        #[soroban_sdk::contracttype]
        enum #r#enum {
            #(#transformed_variants,)*
        }

        #(#storage_functions)*
    };

    let contract_storage_tests = contract_storage_tests(r#enum, input);

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
            quote! { #variant_name }
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            let types = named.iter().map(|f| &f.ty);
            quote! { #variant_name(#(#types),*) }
        }
        _ => panic!("only unit variants or named fields are supported in storage enums"),
    }
}

/// Generates the storage schema tests for a storage enum.
fn contract_storage_tests(r#enum: &Ident, enum_input: &DeriveInput) -> TokenStream {
    let test_module = format_ident!(
        "{}_storage_layout_tests",
        r#enum.to_string().to_snake_case()
    );

    let test = format_ident!(
        "ensure_{}_storage_schema_is_unchanged",
        r#enum.to_string().to_snake_case()
    );

    let enum_file: syn::File = syn::parse2(quote! { #enum_input }).unwrap();
    let formatted_enum = unparse(&enum_file)
        .replace("    #[instance]", "\n    #[instance]")
        .replace("    #[persistent]", "\n    #[persistent]")
        .replace("    #[temporary]", "\n    #[temporary]");

    quote! {
        #[cfg(test)]
        mod #test_module {
            use goldie;

            #[test]
            fn #test() {
                goldie::assert!(#formatted_enum);
            }
        }
    }
}

/// Tests the storage schema generation for a storage enum.
#[cfg(test)]
mod tests {

    #[test]
    fn storage_schema_generation_succeeds() {
        let enum_input: syn::DeriveInput = syn::parse_quote! {
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

        let storage_module = crate::storage::contract_storage(&enum_input);
        let storage_module_file: syn::File = syn::parse2(storage_module).unwrap();
        let formatted_storage_module = prettyplease::unparse(&storage_module_file)
            .replace("pub fn ", "\npub fn ")
            .replace("#[cfg(test)]", "\n#[cfg(test)]");
        goldie::assert!(formatted_storage_module);
    }

    #[test]
    #[should_panic(expected = "contractstorage can only be used on enums")]
    fn non_enum_fails() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct NotAnEnum {
                field: u32,
            }
        };

        crate::storage::contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "only unit variants or named fields are supported in storage enums")]
    fn tuple_variant_fails() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value(u32)]
                TupleVariant(String, u32),
            }
        };

        crate::storage::contract_storage(&input);
    }

    #[test]
    #[should_panic(
        expected = "storage type must be specified exactly once as 'instance', 'persistent', or 'temporary'"
    )]
    fn missing_storage_type_fails() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[value(u32)]
                Counter,
            }
        };

        crate::storage::contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "exactly one of #[status] and #[value(Type)] must be provided")]
    fn missing_value_and_status_attribute_fails() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                Counter,
            }
        };

        crate::storage::contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "only unit variants or named fields are supported in storage enums")]
    fn fields_names_tuple_variant_fails() {
        use super::FieldsExt;
        let fields = syn::Fields::Unnamed(syn::parse_quote! {
            (String, u32)
        });

        fields.names();
    }

    #[test]
    #[should_panic(expected = "only unit variants or named fields are supported in storage enums")]
    fn fields_types_tuple_variant_fails() {
        use super::FieldsExt;
        let fields = syn::Fields::Unnamed(syn::parse_quote! {
            (String, u32)
        });

        fields.types();
    }

    #[test]
    #[should_panic(expected = "exactly one of #[status] and #[value(Type)] must be provided")]
    fn status_and_value_fails() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value(bool)]
                #[status]
                InvalidKey,
            }
        };

        crate::storage::contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "failed to parse value type")]
    fn invalid_value_type_fails() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value(!@$Type)]
                InvalidKey,
            }
        };

        crate::storage::contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "value attribute must contain a type parameter: #[value(Type)]")]
    fn value_without_type_fails() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value]
                InvalidKey,
            }
        };

        crate::storage::contract_storage(&input);
    }

    #[test]
    #[should_panic(
        expected = "contractstorage can only be used on private enums (remove 'pub' keyword)"
    )]
    fn public_enum_fails() {
        let input: syn::DeriveInput = syn::parse_quote! {
            pub enum PublicEnum {
                #[instance]
                #[value(u32)]
                Counter,
            }
        };

        crate::storage::contract_storage(&input);
    }
}
