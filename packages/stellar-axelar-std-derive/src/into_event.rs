use heck::ToSnakeCase;
use proc_macro2::Ident;
use quote::quote;
use syn::{DeriveInput, LitStr, Type};

pub fn into_event(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let event_name = event_name_snake_case(input);
    let ((topic_field_idents, topic_types), (data_field_idents, data_types)) =
        event_struct_fields(input);

    let topic_type_tokens = topic_types.iter().map(|ty| quote!(#ty));
    let data_type_tokens = data_types.iter().map(|ty| quote!(#ty));

    let emit_impl = quote! {
        fn emit(self, env: &soroban_sdk::Env) {
            use soroban_sdk::IntoVal;

            let topics = (
                soroban_sdk::Symbol::new(env, #event_name),
                #(soroban_sdk::IntoVal::<soroban_sdk::Env, soroban_sdk::Val>::into_val(&self.#topic_field_idents, env),)*
            );

            let data: soroban_sdk::Vec<soroban_sdk::Val> = soroban_sdk::vec![
                env
                #(, soroban_sdk::IntoVal::<_, soroban_sdk::Val>::into_val(&self.#data_field_idents, env))*
            ];

            env.events().publish(topics, data);
        }
    };

    let from_event_impl = quote! {
        fn from_event(env: &soroban_sdk::Env, topics: soroban_sdk::Vec<soroban_sdk::Val>, data: soroban_sdk::Val) -> Self {
            use soroban_sdk::TryFromVal;

            // Verify the event name matches
            let event_name = soroban_sdk::Symbol::try_from_val(env, &topics.get(0)
                .expect("missing event name in topics"))
                .expect("invalid event name type");
            assert_eq!(event_name, soroban_sdk::Symbol::new(env, #event_name), "event name mismatch");

            // Parse topics from Val to the corresponding type,
            // and assign them to a variable with the same name as the struct field
            // E.g. let destination_chain = String::try_from_val(env, &topics.get(1));
            // Start from index 1 because the first topic is the event name
            let mut topic_idx = 1;
            #(
                let #topic_field_idents = <#topic_type_tokens>::try_from_val(env, &topics.get(topic_idx)
                    .expect("the number of topics does not match this function's definition"))
                    .expect("given topic value does not match the expected type");
                topic_idx += 1;
            )*

            // Parse data from Val to the corresponding types,
            // and assign them to a variable with the same name as the struct field
            // E.g. let message = Message::try_from_val(env, &data.get(0));
            // `data` is required to be a `Vec<Val>`
            let data = soroban_sdk::Vec::<soroban_sdk::Val>::try_from_val(env, &data)
                .expect("invalid data format");

            let mut data_idx = 0;
            #(
                let #data_field_idents = <#data_type_tokens>::try_from_val(env, &data.get(data_idx)
                    .expect("the number of data entries does not match this function's definition"))
                    .expect("given data value does not match the expected type");
                data_idx += 1;
            )*

            // Construct the struct from the parsed topics and data.
            // Since the variables created above have the same name as the struct fields,
            // the compiler will automatically assign the values to the struct fields.
            Self {
                #(#topic_field_idents,)*
                #(#data_field_idents,)*
            }
        }
    };

    let schema_impl = quote! {
        fn schema(env: &soroban_sdk::Env) -> &'static str {
            concat!(
                #event_name, " {\n",
                #(
                    "    #[topic] ",
                    stringify!(#topic_field_idents),
                    ": ",
                    stringify!(#topic_types),
                    ",\n",
                )*
                #(
                    "    #[data]  ",
                    stringify!(#data_field_idents),
                    ": ",
                    stringify!(#data_types),
                    ",\n",
                )*
                "}"
            )
        }
    };

    quote! {
        impl stellar_axelar_std::events::Event for #name {
            #emit_impl

            #from_event_impl

            #schema_impl
        }
    }
}

fn event_name_snake_case(input: &DeriveInput) -> String {
    input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("event_name"))
        .map(|attr| attr.parse_args::<LitStr>().unwrap().value())
        .unwrap_or_else(|| {
            input
                .ident
                .to_string()
                .strip_suffix("Event")
                .unwrap()
                .to_snake_case()
        })
}

type EventIdent<'a> = Vec<&'a Ident>;
type EventType<'a> = Vec<&'a Type>;
type EventStructFields<'a> = (EventIdent<'a>, EventType<'a>);

fn event_struct_fields(input: &DeriveInput) -> (EventStructFields, EventStructFields) {
    let syn::Data::Struct(data_struct) = &input.data else {
        panic!("IntoEvent can only be derived for structs");
    };

    let mut topic_idents = Vec::new();
    let mut topic_types = Vec::new();
    let mut data_idents = Vec::new();
    let mut data_types = Vec::new();

    for field in data_struct.fields.iter() {
        if let Some(ident) = field.ident.as_ref() {
            if field.attrs.iter().any(|attr| attr.path().is_ident("data")) {
                data_idents.push(ident);
                data_types.push(&field.ty);
            } else {
                topic_idents.push(ident);
                topic_types.push(&field.ty);
            }
        }
    }

    ((topic_idents, topic_types), (data_idents, data_types))
}
