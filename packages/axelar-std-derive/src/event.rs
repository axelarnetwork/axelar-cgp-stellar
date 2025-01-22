use heck::ToSnakeCase;
use proc_macro2::Ident;
use quote::quote;
use syn::{DeriveInput, LitStr, Type};

pub fn derive_event_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let event_name = event_name_snake_case(input);
    let ((topic_idents, _), (data_idents, _)) = event_struct_fields(input);

    let data_impl = quote! {
        fn data(&self, env: &soroban_sdk::Env) -> impl soroban_sdk::IntoVal<soroban_sdk::Env, soroban_sdk::Val> + core::fmt::Debug {
            let data: soroban_sdk::Vec<soroban_sdk::Val> = soroban_sdk::vec![
                env
                #(, soroban_sdk::IntoVal::<_, soroban_sdk::Val>::into_val(&self.#data_idents, env))*
            ];
            data
        }
    };

    quote! {
        impl stellar_axelar_std::events::Event for #name {
            fn topics(&self, env: &soroban_sdk::Env) -> impl soroban_sdk::Topics + core::fmt::Debug {
                (
                    soroban_sdk::Symbol::new(env, #event_name),
                    #(soroban_sdk::IntoVal::<soroban_sdk::Env, soroban_sdk::Val>::into_val(&self.#topic_idents, env),)*
                )
            }

            #data_impl

            fn emit(self, env: &soroban_sdk::Env) {
                env.events().publish(self.topics(env), self.data(env));
            }
        }
    }
}

#[cfg(any(test, feature = "testutils"))]
pub fn derive_event_testutils_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let ((_, topic_types), (_, data_types)) = event_struct_fields(input);
    event_testutils(name, topic_types, data_types)
}

#[cfg(any(test, feature = "testutils"))]
fn event_testutils(
    name: &Ident,
    topic_types: Vec<&Type>,
    data_types: Vec<&Type>,
) -> proc_macro2::TokenStream {
    quote! {
        impl stellar_axelar_std::events::EventTestutils for #name {
            fn matches(self, env: &soroban_sdk::Env, event: &(soroban_sdk::Address, soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val)) -> bool {
                use soroban_sdk::IntoVal;
                use stellar_axelar_std::events::Event;

                Self::standardized_fmt(env, event) == Self::standardized_fmt(env, &(event.0.clone(), self.topics(env).into_val(env), self.data(env).into_val(env)))
            }

            #[allow(unused_assignments)]
            #[allow(unused_variables)]
            #[allow(unused_mut)]
            fn standardized_fmt(env: &soroban_sdk::Env, (contract_id, topics, data): &(soroban_sdk::Address, soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val)) -> std::string::String {
                use soroban_sdk::TryFromVal;

                let mut topics_output: std::vec::Vec<std::string::String> = std::vec![];

                let event_name = topics.get(0).expect("event name topic missing");
                topics_output.push(std::format!("{:?}", soroban_sdk::Symbol::try_from_val(env, &event_name)
                    .expect("event name should be a Symbol")));

                let mut i = 1;
                #(
                    let topic = topics.get(i).expect("the number of topics does not match this function's definition");
                    topics_output.push(std::format!("{:?}", <#topic_types>::try_from_val(env, &topic)
                        .expect("given topic value does not match the expected type")));

                    i += 1;
                )*

                let data = soroban_sdk::Vec::<soroban_sdk::Val>::try_from_val(env, data)
                    .expect("data should be defined as a vector-compatible type");

                let mut data_output: std::vec::Vec<std::string::String> = std::vec![];

                let mut i = 0;
                #(
                    let data_entry = data.get(i).expect("the number of data entries does not match this function's definition");
                    data_output.push(std::format!("{:?}", <#data_types>::try_from_val(env, &data_entry)
                        .expect("given data value does not match the expected type")));

                    i += 1;
                )*

                std::format!("contract: {:?}\ntopics: ({})\ndata: ({})",
                    contract_id,
                    topics_output.join(", "),
                    data_output.join(", ")
                )
            }
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
