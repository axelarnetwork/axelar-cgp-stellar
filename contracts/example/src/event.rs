extern crate std;

use soroban_sdk::{Address, Bytes, BytesN, String};
use stellar_axelar_std::IntoEvent;

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct ExecutedEvent {
    pub source_chain: String,
    pub message_id: String,
    pub source_address: String,
    #[data]
    pub payload: Bytes,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct TokenReceivedEvent {
    pub source_chain: String,
    pub message_id: String,
    pub source_address: Bytes,
    #[data]
    pub payload: Bytes,
    #[data]
    pub token_id: BytesN<32>,
    #[data]
    pub token_address: Address,
    #[data]
    pub amount: i128,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct TokenSentEvent {
    pub sender: Address,
    pub token_id: BytesN<32>,
    pub destination_chain: String,
    #[data]
    pub destination_address: Bytes,
    #[data]
    pub amount: i128,
    #[data]
    pub message: Option<Bytes>,
}
