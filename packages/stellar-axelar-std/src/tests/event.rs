use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate as stellar_axelar_std;
use crate::events::{fmt_last_emitted_event, Event};
use crate::IntoEvent;

#[contract]
pub struct Contract;

#[derive(Debug, PartialEq, Eq, IntoEvent)]
struct EmptyEvent;

#[derive(Debug, PartialEq, Eq, IntoEvent)]
struct SingleDataEvent {
    topic1: Address,
    topic2: Address,
    #[data]
    data: String,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
#[event_name("custom_name")]
struct NamedEvent {
    topic: Address,
    #[data]
    data: String,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
struct MultiDataEvent {
    topic1: String,
    topic2: Address,
    #[data]
    data1: String,
    #[data]
    data2: String,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
struct NoDataEvent {
    topic1: String,
    topic2: Address,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
struct NoTopicEvent {
    #[data]
    data1: String,
    #[data]
    data2: Address,
    #[data]
    data3: String,
}

#[contractimpl]
impl Contract {
    pub const fn __constructor() {}

    pub fn empty(env: &Env) {
        EmptyEvent.emit(env);
    }

    pub fn single_data(env: &Env, topic1: Address, topic2: Address, data: String) {
        SingleDataEvent {
            topic1,
            topic2,
            data,
        }
        .emit(env);
    }

    pub fn named(env: &Env, topic: Address, data: String) {
        NamedEvent { topic, data }.emit(env);
    }

    pub fn multi_data(env: &Env, topic1: String, topic2: Address, data1: String, data2: String) {
        MultiDataEvent {
            topic1,
            topic2,
            data1,
            data2,
        }
        .emit(env);
    }

    pub fn no_data(env: &Env, topic1: String, topic2: Address) {
        NoDataEvent { topic1, topic2 }.emit(env);
    }

    pub fn no_topic(env: &Env, data1: String, data2: Address, data3: String) {
        NoTopicEvent {
            data1,
            data2,
            data3,
        }
        .emit(env);
    }
}

#[test]
fn event_empty_succeeds() {
    let env = &Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    client.empty();
    goldie::assert!(fmt_last_emitted_event::<EmptyEvent>(env));
}

#[test]
fn single_data_event_emitted() {
    let env = &Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let topic1 = Address::generate(env);
    let topic2 = Address::generate(env);
    let data = String::from_str(env, "100");

    client.single_data(&topic1, &topic2, &data);
    goldie::assert!(fmt_last_emitted_event::<SingleDataEvent>(env));
}

#[test]
fn named_event_emitted() {
    let env = &Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let topic = Address::generate(env);
    let data = String::from_str(env, "100");

    client.named(&topic, &data);
    goldie::assert!(fmt_last_emitted_event::<NamedEvent>(env));
}

#[test]
fn multi_data_event_emitted() {
    let env = &Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let topic1 = String::from_str(env, "topic-1");
    let topic2 = Address::generate(env);
    let data1 = String::from_str(env, "data-1");
    let data2 = String::from_str(env, "data-2");

    client.multi_data(&topic1, &topic2, &data1, &data2);
    goldie::assert!(fmt_last_emitted_event::<MultiDataEvent>(env));
}

#[test]
fn no_data_event_emitted() {
    let env = &Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let topic1 = String::from_str(env, "topic-1");
    let topic2 = Address::generate(env);

    client.no_data(&topic1, &topic2);
    goldie::assert!(fmt_last_emitted_event::<NoDataEvent>(env));
}

#[test]
fn no_topic_event_emitted() {
    let env = &Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let data1 = String::from_str(env, "data-1");
    let data2 = Address::generate(env);
    let data3 = String::from_str(env, "data-3");

    client.no_topic(&data1, &data2, &data3);
    goldie::assert!(fmt_last_emitted_event::<NoTopicEvent>(env));
}
