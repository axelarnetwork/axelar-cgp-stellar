use soroban_sdk::{Address, Env, Symbol};

pub fn add_minter(env: &Env, minter: Address) {
    let topics = (Symbol::new(env, "minter_added"), minter);
    env.events().publish(topics, ());
}

pub fn remove_minter(env: &Env, minter: Address) {
    let topics = (Symbol::new(env, "minter_removed"), minter);
    env.events().publish(topics, ());
}
