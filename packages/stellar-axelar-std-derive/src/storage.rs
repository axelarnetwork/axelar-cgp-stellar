use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, FieldsNamed, Meta, Type, Variant};

struct StorageAttributes {
    storage_type: StorageType,
    value_type: Type,
}

#[derive(Debug)]
enum StorageType {
    Instance,
    Persistent,
    Temporary,
}

impl StorageType {
    pub fn storage_method(&self) -> TokenStream {
        match self {
            Self::Instance => quote! { instance },
            Self::Persistent => quote! { persistent },
            Self::Temporary => quote! { temporary },
        }
    }

    pub fn ttl_method(&self) -> TokenStream {
        match self {
            Self::Persistent => quote! {
                stellar_axelar_std::ttl::extend_persistent_ttl(env, &key);
            },
            Self::Instance => quote! {
                stellar_axelar_std::ttl::extend_instance_ttl(env);
            },
            Self::Temporary => quote! {},
        }
    }
}

/// Generates the storage enum and its associated functions.
pub fn contractstorage(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        panic!("contractstorage can only be used on enums.");
    };

    let transformed_variants: Vec<_> = variants.iter().map(transform_variant).collect();

    let storage_fns: Vec<_> = variants
        .iter()
        .map(|variant| {
            let storage_type = storage_type(&variant.attrs);
            let value_type = value_type(&variant.attrs);
            storage_fns(
                name,
                variant,
                &StorageAttributes {
                    storage_type,
                    value_type,
                },
            )
        })
        .collect();

    let public_fns: Vec<_> = variants
        .iter()
        .map(|variant| {
            let value_type = value_type(&variant.attrs);
            public_storage_fns(name, variant, &value_type)
        })
        .collect();

    let output = quote! {
        #[contracttype]
        enum #name {
            #(#transformed_variants,)*
        }

        impl #name {
            #(#storage_fns)*
        }

        #(#public_fns)*
    };

    output
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
/// ```
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
        _ => panic!("Only unit variants or named fields are supported in storage enums."),
    }
}

/// Returns the storage type of a storage enum variant.
fn storage_type(attrs: &[Attribute]) -> StorageType {
    attrs.iter().flat_map(|attr| match attr {
        _ if attr.path().is_ident("instance") => Some(StorageType::Instance),
        _ if attr.path().is_ident("persistent") => Some(StorageType::Persistent),
        _ if attr.path().is_ident("temporary") => Some(StorageType::Temporary),
        _ => None})
    .exactly_one()
    .expect("Storage type must be specified exactly once as 'instance', 'persistent', or 'temporary'.")
}

/// Returns the value type of a storage enum variant.
fn value_type(attrs: &[Attribute]) -> Type {
    attrs
        .iter()
        .flat_map(|attr| {
            if attr.path().is_ident("value") {
                if let Meta::List(list) = &attr.meta {
                    Some(
                        list.parse_args::<Type>()
                            .expect("Failed to parse value type."),
                    )
                } else {
                    panic!("Value attribute must contain a type parameter: #[value(Type)]");
                }
            } else {
                None
            }
        })
        .exactly_one()
        .expect("Missing required #[value(Type)] attribute.")
}

/// Generates the storage getter, setter, and deleter functions for a storage enum variant.
fn storage_fns(
    enum_name: &Ident,
    variant: &Variant,
    storage_attrs: &StorageAttributes,
) -> TokenStream {
    let variant_ident = &variant.ident;

    let (field_names, field_types) = fields_data(&variant.fields);

    let value_type = storage_attrs.value_type.clone();

    let (getter_name, setter_name, deleter_name) = fn_names(variant);

    let storage_method = storage_attrs.storage_type.storage_method();
    let ttl_fn = storage_attrs.storage_type.ttl_method();

    let key = if field_names.is_empty() {
        quote! { #enum_name::#variant_ident }
    } else {
        quote! { #enum_name::#variant_ident(#(#field_names),*) }
    };

    let param_list = if field_names.is_empty() {
        quote! { env: &soroban_sdk::Env }
    } else {
        quote! { env: &soroban_sdk::Env, #(#field_names: #field_types),* }
    };

    quote! {
        pub fn #getter_name(#param_list) -> Option<#value_type> {
            let key = #key;
            let value = env.storage()
                .#storage_method()
                .get::<_, #value_type>(&key);

            if value.is_some() {
                #ttl_fn
            }

            value
        }

        pub fn #setter_name(#param_list, value: &#value_type) {
            let key = #key;

            env.storage()
                .#storage_method()
                .set(&key, value);

            #ttl_fn
        }

        pub fn #deleter_name(#param_list) {
            let key = #key;
            env.storage()
                .#storage_method()
                .remove(&key);
        }
    }
}

/// Generates the public module-level storage functions.
fn public_storage_fns(enum_name: &Ident, variant: &Variant, value_type: &Type) -> TokenStream {
    let (getter_name, setter_name, deleter_name) = fn_names(variant);

    let (field_names, field_types) = fields_data(&variant.fields);

    let param_list = if field_names.is_empty() {
        quote! { env: &soroban_sdk::Env }
    } else {
        quote! { env: &soroban_sdk::Env, #(#field_names: #field_types),* }
    };

    let fn_args = if field_names.is_empty() {
        quote! { env }
    } else {
        quote! { env, #(#field_names),* }
    };

    quote! {
        pub fn #getter_name(#param_list) -> Option<#value_type> {
            #enum_name::#getter_name(#fn_args)
        }

        pub fn #setter_name(#param_list, value: &#value_type) {
            #enum_name::#setter_name(#fn_args, value)
        }

        pub fn #deleter_name(#param_list) {
            #enum_name::#deleter_name(#fn_args)
        }
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
        _ => panic!("Only unit variants or named fields are supported in storage enums."),
    }
}

fn fn_names(variant: &Variant) -> (Ident, Ident, Ident) {
    (
        format_ident!("{}", variant.ident.to_string().to_snake_case()),
        format_ident!("set_{}", variant.ident.to_string().to_snake_case()),
        format_ident!("delete_{}", variant.ident.to_string().to_snake_case()),
    )
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn storage_schema_generation_succeeds() {
        let input: DeriveInput = syn::parse_quote! {
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
            }
        };

        let generated = contractstorage(&input);
        let file: syn::File = syn::parse2(generated).unwrap();
        let formatted = prettyplease::unparse(&file);
        goldie::assert!(formatted);
    }

    #[test]
    #[should_panic(expected = "contractstorage can only be used on enums.")]
    fn non_enum_fails() {
        let input: DeriveInput = syn::parse_quote! {
            struct NotAnEnum {
                field: u32,
            }
        };

        contractstorage(&input);
    }

    #[test]
    #[should_panic(expected = "Only unit variants or named fields are supported in storage enums.")]
    fn tuple_variant_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value(u32)]
                TupleVariant(String, u32),
            }
        };

        contractstorage(&input);
    }

    #[test]
    #[should_panic(
        expected = "Storage type must be specified exactly once as 'instance', 'persistent', or 'temporary'."
    )]
    fn missing_storage_type_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[value(u32)]
                Counter,
            }
        };

        contractstorage(&input);
    }

    #[test]
    #[should_panic(expected = "Value attribute must contain a type parameter: #[value(Type)]")]
    fn missing_value_type_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value]
                Counter,
            }
        };

        contractstorage(&input);
    }

    #[test]
    #[should_panic(expected = "Missing required #[value(Type)] attribute.")]
    fn missing_value_attribute_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                Counter,
            }
        };

        contractstorage(&input);
    }

    #[test]
    #[should_panic(expected = "Only unit variants or named fields are supported in storage enums.")]
    fn fields_data_tuple_variant_fails() {
        let fields = Fields::Unnamed(syn::parse_quote! {
            (String, u32)
        });

        fields_data(&fields);
    }
}
