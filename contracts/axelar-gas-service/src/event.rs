use soroban_sdk::{Address, Bytes, Env, String, Symbol};
use stellar_axelar_std::types::Token;

pub fn gas_paid(
    env: &Env,
    sender: Address,
    destination_chain: String,
    destination_address: String,
    payload: Bytes,
    spender: Address,
    token: Token,
    metadata: Bytes,
) {
    let topics = (
        Symbol::new(env, "gas_paid"),
        sender,
        destination_chain,
        destination_address,
        env.crypto().keccak256(&payload),
        spender,
        token,
    );
    env.events().publish(topics, (metadata,));
}

pub fn gas_added(env: &Env, sender: Address, message_id: String, spender: Address, token: Token) {
    let topics = (
        Symbol::new(env, "gas_added"),
        sender,
        message_id,
        spender,
        token,
    );
    env.events().publish(topics, ());
}

pub fn refunded(env: &Env, message_id: String, receiver: Address, token: Token) {
    let topics = (
        Symbol::new(env, "gas_refunded"),
        message_id,
        receiver,
        token,
    );
    env.events().publish(topics, ());
}

pub fn fee_collected(env: &Env, receiver: Address, token: Token) {
    let topics = (Symbol::new(env, "gas_collected"), receiver, token);
    env.events().publish(topics, ());
}
