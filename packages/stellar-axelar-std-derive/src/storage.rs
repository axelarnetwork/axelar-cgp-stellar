use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, FieldsNamed, Meta, Type, Variant};

enum Value {
    Status,
    Type(Type),
}

struct StorageAttributes {
    storage_type: StorageType,
    value: Value,
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
pub fn contract_storage(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        panic!("contractstorage can only be used on enums.");
    };

    let transformed_variants: Vec<_> = variants.iter().map(transform_variant).collect();

    let public_fns: Vec<_> = variants
        .iter()
        .map(|variant| {
            public_storage_fns(
                name,
                variant,
                &StorageAttributes {
                    storage_type: storage_type(&variant.attrs),
                    value: value(&variant.attrs),
                },
            )
        })
        .collect();

    let contract_storage = quote! {
        #[contracttype]
        enum #name {
            #(#transformed_variants,)*
        }

        #(#public_fns)*
    };

    let contract_storage_tests = contract_storage_tests(name, &contract_storage);

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
        _ => panic!("Only unit variants or named fields are supported in storage enums."),
    }
}

/// Returns the storage type of a storage enum variant.
fn storage_type(attrs: &[Attribute]) -> StorageType {
    attrs.iter().flat_map(|attr| match attr {
        _ if attr.path().is_ident("instance") => Some(StorageType::Instance),
        _ if attr.path().is_ident("persistent") => Some(StorageType::Persistent),
        _ if attr.path().is_ident("temporary") => Some(StorageType::Temporary),
        _ => None,})
    .exactly_one()
    .expect("Storage type must be specified exactly once as 'instance', 'persistent', or 'temporary'.")
}

/// Returns the status xor value type of a storage enum variant.
fn value(attrs: &[Attribute]) -> Value {
    let has_status = attrs.iter().any(|attr| attr.path().is_ident("status"));
    let value_attrs: Vec<_> = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("value"))
        .collect();

    match (has_status, !value_attrs.is_empty()) {
        (true, false) => Value::Status,
        (false, true) => {
            let attr = value_attrs[0];
            if let Meta::List(list) = &attr.meta {
                Value::Type(
                    list.parse_args::<Type>()
                        .expect("Failed to parse value type."),
                )
            } else {
                panic!("Value attribute must contain a type parameter: #[value(Type)]");
            }
        }
        (false, false) => panic!("Missing required attribute: either #[status] xor #[value(Type)]"),
        _ => panic!("A storage key cannot have both #[status] and #[value] attributes."),
    }
}

/// Generates the public module-level storage functions.
fn public_storage_fns(
    enum_name: &Ident,
    variant: &Variant,
    StorageAttributes {
        storage_type,
        value,
    }: &StorageAttributes,
) -> TokenStream {
    let variant_ident = &variant.ident;

    let (field_names, field_types) = fields_data(&variant.fields);
    let (getter_name, setter_name, remover_name) = fn_names(variant);

    let storage_method = storage_type.storage_method();
    let ttl_fn = storage_type.ttl_method();

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

    match &value {
        Value::Status => quote! {
            pub fn #getter_name(#param_list) -> bool {
                let key = #key;
                env.storage()
                    .#storage_method()
                    .has(&key)
            }

            pub fn #setter_name(#param_list) {
                let key = #key;
                env.storage()
                    .#storage_method()
                    .set(&key, &());
            }

            pub fn #remover_name(#param_list) {
                let key = #key;
                env.storage()
                    .#storage_method()
                    .remove(&key);
            }
        },
        Value::Type(value_type) => quote! {
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

            pub fn #remover_name(#param_list) {
                let key = #key;
                env.storage()
                    .#storage_method()
                    .remove(&key);
            }
        },
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
        format_ident!("remove_{}", variant.ident.to_string().to_snake_case()),
    )
}

fn contract_storage_tests(enum_name: &Ident, storage_enum: &TokenStream) -> TokenStream {
    let test_module_name = format_ident!(
        "{}_storage_layout_tests",
        enum_name.to_string().to_snake_case()
    );

    let test_name = format_ident!(
        "ensure_{}_storage_schema_is_unchanged",
        enum_name.to_string().to_snake_case()
    );

    quote! {
        #[cfg(test)]
        mod #test_module_name {
            #[test]
            fn #test_name() {
                goldie::assert!(stringify!(#storage_enum));
            }
        }
    }
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

                #[instance]
                #[status]
                Initialized,

                #[persistent]
                #[status]
                Paused,
            }
        };

        let generated = contract_storage(&input);
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

        contract_storage(&input);
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

        contract_storage(&input);
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

        contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "Value attribute must contain a type parameter: #[value(Type)]")]
    fn missing_value_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                #[value]
                Counter,
            }
        };

        contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "Missing required attribute: either #[status] xor #[value(Type)]")]
    fn missing_value_attribute_fails() {
        let input: DeriveInput = syn::parse_quote! {
            enum InvalidEnum {
                #[instance]
                Counter,
            }
        };

        contract_storage(&input);
    }

    #[test]
    #[should_panic(expected = "Only unit variants or named fields are supported in storage enums.")]
    fn fields_data_tuple_variant_fails() {
        let fields = Fields::Unnamed(syn::parse_quote! {
            (String, u32)
        });

        fields_data(&fields);
    }

    #[test]
    #[should_panic(expected = "A storage key cannot have both #[status] and #[value] attributes.")]
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
}
