use core::fmt::Debug;

use soroban_sdk::{Env, IntoVal, Topics, Val, Vec};
#[cfg(any(test, feature = "testutils"))]
pub use testutils::*;

pub trait Event: Debug + PartialEq + Sized {
    fn topics(&self, env: &Env) -> impl Topics + Debug;

    /// A default empty tuple/vector is used for event data, since majority of events only use topics.
    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        Vec::<Val>::new(env)
    }

    fn emit(self, env: &Env) {
        env.events().publish(self.topics(env), self.data(env));
    }
}

#[cfg(any(test, feature = "testutils"))]
mod testutils {
    use soroban_sdk::testutils::Events;
    use soroban_sdk::{Address, Env, Val, Vec};

    use crate::events::Event;

    pub trait EventTestutils: Event {
        fn matches(self, env: &Env, event: &(Address, Vec<Val>, Val)) -> bool;

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
}
