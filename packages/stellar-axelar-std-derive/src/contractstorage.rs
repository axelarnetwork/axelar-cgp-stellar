use heck::ToSnakeCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, FieldsNamed, Meta, Type, Variant};

#[derive(Default)]
struct StorageAttributes {
    storage_type: StorageType,
    value_type: Option<Type>,
}

#[derive(Default, Debug)]
enum StorageType {
    #[default]
    Instance,
    Persistent,
    Temporary,
}

pub fn contractstorage(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        panic!("contractstorage can only be used on enums.");
    };

    let transformed_variants: Vec<_> = variants
        .iter()
        .map(|variant| transform_variant(variant))
        .collect();

    let storage_fns: Vec<_> = variants
        .iter()
        .map(|variant| {
            let attrs = storage_attributes(&variant.attrs);
            storage_fns(name, variant, &attrs)
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

fn storage_attributes(attrs: &[Attribute]) -> StorageAttributes {
    let mut storage_attributes = StorageAttributes::default();

    for attr in attrs {
        let path_str = attr.path().to_token_stream().to_string();

        match path_str.as_str() {
            "instance" => {
                storage_attributes.storage_type = StorageType::Instance;
            }
            "persistent" => {
                storage_attributes.storage_type = StorageType::Persistent;
            }
            "temporary" => {
                storage_attributes.storage_type = StorageType::Temporary;
            }
            "value" => {
                if let Meta::List(list) = &attr.meta {
                    if let Ok(ty) = list.parse_args::<Type>() {
                        storage_attributes.value_type = Some(ty);
                    }
                }
            }
            _ => {}
        }
    }

    storage_attributes
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
            use stellar_axelar_std::ttl::extend_instance_ttl;
            extend_instance_ttl(env);
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
