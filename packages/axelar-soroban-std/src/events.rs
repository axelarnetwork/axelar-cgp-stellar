use core::fmt::Debug;
use soroban_sdk::testutils::Events;
use soroban_sdk::{Address, Env, IntoVal, Topics, TryFromVal, Val, Vec};

pub trait Event: TryFromVal<Env, (Vec<Val>, Val)> + Debug {
    fn topic() -> impl Topics;
    fn data(&self) -> impl IntoVal<Env, Val>;

    fn emit(&self, env: &Env) {
        env.events().publish(Self::topic(), self.data());
    }
}

pub fn match_last_emitted_event<E>(env: &Env) -> Option<(Address, E)>
where
    E: Event,
{
    env.events()
        .all()
        .last()
        .and_then(|(contract_id, topics, data)| {
            E::try_from_val(env, &(topics, data))
                .ok()
                .map(|e| (contract_id, e))
        })
}

pub fn match_emitted_event_at_idx<E>(env: &Env, idx: u32) -> Option<(Address, E)>
where
    E: Event,
{
    env.events()
        .all()
        .get(idx)
        .and_then(|(contract_id, topics, data)| {
            E::try_from_val(env, &(topics, data))
                .ok()
                .map(|e| (contract_id, e))
        })
}

#[cfg(test)]
mod test {}
