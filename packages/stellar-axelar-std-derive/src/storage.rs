use heck::ToSnakeCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, FieldsNamed, Meta, Type, Variant};

struct StorageAttributes {
    storage_type: StorageType,
    value_type: Option<Type>,
}

enum StorageType {
    Instance,
    Persistent,
    Temporary,
}

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

    let output = quote! {
        #[contracttype]
        pub enum #name {
            #(#transformed_variants,)*
        }

        impl #name {
            #(#storage_fns)*
        }
    };

    output
}

/// Transforms a contractstorage enum variant with named fields into a storage key map, or a single storage key for a unit variant.
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
///     Gateway, // Unit variant
///
///     #[temporary]
///     #[value(Address)]
///     Users { user: Address }, // Named variant
/// }
/// ```
///
/// /* Generated */
/// #[contracttype]
/// pub enum DataKey {
///     Gateway, // Non-map
///     Users(Address), // Map
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

fn storage_type(attrs: &[Attribute]) -> StorageType {
    let mut found_type = None;

    for attr in attrs {
        let path_str = attr.path().to_token_stream().to_string();
        match path_str.as_str() {
            "instance" | "persistent" | "temporary" => {
                if found_type.is_some() {
                    panic!("Multiple storage types specified - must have exactly one of: 'instance', 'persistent', or 'temporary'.");
                }
                found_type = Some(match path_str.as_str() {
                    "instance" => StorageType::Instance,
                    "persistent" => StorageType::Persistent,
                    "temporary" => StorageType::Temporary,
                    _ => unreachable!(),
                });
            }
            "value" => continue,
            unknown => panic!("Unknown storage attribute: {}", unknown),
        }
    }

    found_type.expect(
        "Storage type must be specified exactly once as 'instance', 'persistent', or 'temporary'.",
    )
}

fn value_type(attrs: &[Attribute]) -> Option<Type> {
    let mut found_type = None;

    for attr in attrs {
        let path_str = attr.path().to_token_stream().to_string();
        if path_str == "value" {
            if found_type.is_some() {
                panic!("multiple value types specified - must have at most one 'value' attribute");
            }
            if let Meta::List(list) = &attr.meta {
                if let Ok(ty) = list.parse_args::<Type>() {
                    found_type = Some(ty);
                }
            }
        }
    }

    found_type
}

fn storage_fns(
    enum_name: &Ident,
    variant: &Variant,
    storage_attrs: &StorageAttributes,
) -> TokenStream {
    let variant_ident = &variant.ident;

    let (field_names, field_types) = match &variant.fields {
        Fields::Unit => (vec![], vec![]),
        Fields::Named(fields) => {
            let names = fields.named.iter().map(|f| &f.ident).collect();
            let types = fields.named.iter().map(|f| &f.ty).collect();
            (names, types)
        }
        _ => panic!("Only unit variants or named fields are supported in storage enums."),
    };

    let value_type = storage_attrs
        .value_type
        .as_ref()
        .expect("value type required");

    let getter_name = format_ident!("get_{}", variant_ident.to_string().to_snake_case());
    let setter_name = format_ident!("set_{}", variant_ident.to_string().to_snake_case());

    let storage_method = match storage_attrs.storage_type {
        StorageType::Instance => quote! { instance },
        StorageType::Persistent => quote! { persistent },
        StorageType::Temporary => quote! { temporary },
    };

    let ttl_fn = match storage_attrs.storage_type {
        StorageType::Persistent => quote! {
            use stellar_axelar_std::ttl::extend_persistent_ttl;
            extend_persistent_ttl(env, &key);
        },
        StorageType::Instance => quote! {
            stellar_axelar_std::ttl::extend_instance_ttl(env);
        },
        _ => quote! {},
    };

    let key_construction = if field_names.is_empty() {
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
            let key = #key_construction;
            let value = env.storage()
                .#storage_method()
                .get(&key);

            if value.is_some() { #ttl_fn }

            value
        }

        pub fn #setter_name(#param_list, value: &#value_type) {
            let key = #key_construction;

            env.storage()
                .#storage_method()
                .set(&key, value);

            #ttl_fn
        }
    }
}
