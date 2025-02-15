use soroban_sdk::{Address, Bytes, BytesN, String};
use stellar_axelar_std::types::Token;
use stellar_axelar_std::IntoEvent;

#[derive(Debug, PartialEq, IntoEvent)]
pub struct GasPaidEvent {
    pub sender: Address,
    pub destination_chain: String,
    pub destination_address: String,
    pub payload_hash: BytesN<32>,
    pub spender: Address,
    pub token: Token,
    #[data]
    pub metadata: Bytes,
}

#[derive(Debug, PartialEq, IntoEvent)]
pub struct GasAddedEvent {
    pub sender: Address,
    pub message_id: String,
    pub spender: Address,
    pub token: Token,
}

#[derive(Debug, PartialEq, IntoEvent)]
pub struct GasRefundedEvent {
    pub message_id: String,
    pub receiver: Address,
    pub token: Token,
}

#[derive(Debug, PartialEq, IntoEvent)]
pub struct GasCollectedEvent {
    pub receiver: Address,
    pub token: Token,
}
