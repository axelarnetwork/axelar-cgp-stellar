use core::fmt::Debug;
use soroban_sdk::{Env, IntoVal, Topics, Val, Vec};

#[cfg(any(test, feature = "testutils"))]
pub use testutils::*;

pub trait Event: Debug + PartialEq {
    fn topics(&self, env: &Env) -> impl Topics + Debug;

    /// A default empty tuple/vector is used for event data, since majority of events only use topics.
    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        Vec::<Val>::new(env)
    }

    fn emit(&self, env: &Env) {
        env.events().publish(self.topics(env), self.data(env));
    }
}

#[cfg(any(test, feature = "testutils"))]
mod testutils {
    use crate::events::Event;
    use soroban_sdk::testutils::Events;
    use soroban_sdk::{Address, Env, Val, Vec};

    pub trait EventTestutils: Event {
        fn matches(&self, env: &Env, event: &(Address, Vec<Val>, Val)) -> bool;

        fn standardized_fmt(
            env: &Env,
            event: &(soroban_sdk::Address, soroban_sdk::Vec<Val>, Val),
        ) -> std::string::String;
    }

    pub fn fmt_last_emitted_event<E>(env: &Env) -> std::string::String
    where
        E: EventTestutils,
    {
        let event = env.events().all().last().expect("no event found");
        E::standardized_fmt(env, &event)
    }

    pub fn fmt_emitted_event_at_idx<E>(env: &Env, mut idx: i32) -> std::string::String
    where
        E: EventTestutils,
    {
        if idx < 0 {
            idx += env.events().all().len() as i32;
        }

        let event = env
            .events()
            .all()
            .get(idx as u32)
            .expect("no event found at the given index");
        E::standardized_fmt(env, &event)
    }

    #[macro_export]
    macro_rules! impl_event_testutils {
        ($event_type:ty, ($($topic_type:ty),*), ($($data_type:ty),*)) => {
            impl $crate::events::EventTestutils for $event_type {
                fn matches(&self, env: &soroban_sdk::Env, event: &(soroban_sdk::Address, soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val)) -> bool {
                    use soroban_sdk::IntoVal;

                    Self::standardized_fmt(env, event) == Self::standardized_fmt(env, &(event.0.clone(), self.topics(env).into_val(env), self.data(env).into_val(env)))
                }

                #[allow(unused_assignments)]
                #[allow(unused_variables)]
                #[allow(unused_mut)]
                fn standardized_fmt(env: &soroban_sdk::Env, (contract_id, topics, data): &(soroban_sdk::Address, soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val)) -> std::string::String {
                    use soroban_sdk::TryFromVal;

                    let mut topics_output: std::vec::Vec<std::string::String> = std::vec![];

                    let mut i = 0;
                    $(
                        let topic = topics.get(i).expect("the number of topics does not match this function's definition");
                        topics_output.push(std::format!("{:?}", <$topic_type>::try_from_val(env, &topic)
                            .expect("given topic value does not match the expected type")));

                        i += 1;
                    )*

                    let data = soroban_sdk::Vec::<soroban_sdk::Val>::try_from_val(env, data).expect("data should be defined as a vector-compatible type");

                    let mut data_output: std::vec::Vec<std::string::String> = std::vec![];

                    let mut i = 0;
                    $(
                        let data_entry = data.get(i).expect("the number of data entries does not match this function's definition");
                        data_output.push(std::format!("{:?}", <$data_type>::try_from_val(env, &data_entry)
                            .expect("given data value does not match the expected type")));

                        i += 1;
                    )*

                    std::format!("contract: {:?}\ntopics: ({})\ndata: ({})", contract_id, topics_output.join(", "), data_output.join(", "))
                }
            }
        };
    }
}

#[cfg(test)]
mod test {
    use crate::events::{Event, EventTestutils};
    use crate::{events, impl_event_testutils};
    use core::fmt::Debug;
    use soroban_sdk::testutils::Events;
    use soroban_sdk::xdr::Int32;
    use soroban_sdk::{contract, BytesN, Env, IntoVal, String, Symbol, Topics, Val};

    #[derive(Debug, PartialEq, Eq)]
    struct TestEvent {
        topic1: Symbol,
        topic2: String,
        topic3: Int32,
        data1: String,
        data2: BytesN<32>,
    }

    impl Event for TestEvent {
        fn topics(&self, _env: &Env) -> impl Topics + Debug {
            (self.topic1.clone(), self.topic2.clone(), self.topic3)
        }

        fn data(&self, _env: &Env) -> impl IntoVal<Env, Val> + Debug {
            (self.data1.clone(), self.data2.clone())
        }
    }

    impl_event_testutils!(TestEvent, (Symbol, String, Int32), (String, BytesN<32>));

    #[contract]
    struct Contract;

    #[test]
    fn format_last_emitted_event() {
        let env = Env::default();
        let expected = TestEvent {
            topic1: Symbol::new(&env, "topic1"),
            topic2: String::from_str(&env, "topic2"),
            topic3: 10,
            data1: String::from_str(&env, "data1"),
            data2: BytesN::from_array(&env, &[3; 32]),
        };

        let contract = env.register(Contract, ());
        env.as_contract(&contract, || {
            expected.emit(&env);
        });

        assert!(expected.matches(&env, &env.events().all().last().unwrap()));

        goldie::assert!(events::fmt_last_emitted_event::<TestEvent>(&env));
    }
}
