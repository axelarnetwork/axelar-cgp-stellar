use soroban_sdk::{Address, Env, Symbol};

pub fn add_operator(env: &Env, operator: Address) {
    let topics = (Symbol::new(env, "operator_added"), operator);
    env.events().publish(topics, ());
}

pub fn remove_operator(env: &Env, operator: Address) {
    let topics = (Symbol::new(env, "operator_removed"), operator);
    env.events().publish(topics, ());
}
