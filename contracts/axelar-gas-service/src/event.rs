use soroban_sdk::{Address, Bytes, BytesN, String};
use stellar_axelar_std::{types::Token, IntoEvent};

#[derive(Debug, PartialEq, Eq, IntoEvent)]
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

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct GasAddedEvent {
    pub sender: Address,
    pub message_id: String,
    pub spender: Address,
    pub token: Token,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
#[event_name("gas_refunded")]
pub struct RefundedEvent {
    pub message_id: String,
    pub receiver: Address,
    pub token: Token,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
#[event_name("gas_collected")]
pub struct FeeCollectedEvent {
    pub receiver: Address,
    pub token: Token,
}
