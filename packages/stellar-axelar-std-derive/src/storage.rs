use heck::ToSnakeCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, FieldsNamed, Meta, Type, Variant};

struct StorageAttributes {
    storage_type: StorageType,
    value_type: Type,
}

enum StorageType {
    Instance,
    Persistent,
    Temporary,
}

impl StorageType {
    pub fn storage_method(&self) -> TokenStream {
        match self {
            StorageType::Instance => quote! { instance },
            StorageType::Persistent => quote! { persistent },
            StorageType::Temporary => quote! { temporary },
        }
    }

    pub fn ttl_method(&self) -> TokenStream {
        match self {
            StorageType::Persistent => quote! {
                stellar_axelar_std::ttl::extend_persistent_ttl(env, &key);
            },
            StorageType::Instance => quote! {
                stellar_axelar_std::ttl::extend_instance_ttl(env);
            },
            StorageType::Temporary => quote! {},
        }
    }
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

/// Transforms a contractstorage enum variant with named fields into a storage key map (tuple variant), or a single storage key for a unit variant.
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

fn storage_type(attrs: &[Attribute]) -> StorageType {
    for attr in attrs {
        let path_str = attr.path().to_token_stream().to_string();
        match path_str.as_str() {
            "instance" => return StorageType::Instance,
            "persistent" => return StorageType::Persistent,
            "temporary" => return StorageType::Temporary,
            "value" => continue,
            unknown => panic!("Unknown storage attribute: {}", unknown),
        }
    }

    panic!(
        "Storage type must be specified exactly once as 'instance', 'persistent', or 'temporary'."
    )
}

fn value_type(attrs: &[Attribute]) -> Type {
    for attr in attrs {
        let path_str = attr.path().to_token_stream().to_string();
        if path_str == "value" {
            if let Meta::List(list) = &attr.meta {
                return list
                    .parse_args::<Type>()
                    .expect("Failed to parse value type.");
            } else {
                panic!("Value attribute must contain a type parameter: #[value(Type)]");
            }
        }
    }

    panic!("Missing required #[value(Type)] attribute.")
}

fn storage_fns(
    enum_name: &Ident,
    variant: &Variant,
    storage_attrs: &StorageAttributes,
) -> TokenStream {
    let variant_ident = &variant.ident;

    let (field_names, field_types) = fields_data(&variant.fields);

    let value_type = storage_attrs.value_type.clone();

    let getter_name = format_ident!("get_{}", variant_ident.to_string().to_snake_case());
    let setter_name = format_ident!("set_{}", variant_ident.to_string().to_snake_case());

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

            if value.is_some() { #ttl_fn }

            value
        }

        pub fn #setter_name(#param_list, value: &#value_type) {
            let key = #key;

            env.storage()
                .#storage_method()
                .set(&key, value);

            #ttl_fn
        }
    }
}

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
