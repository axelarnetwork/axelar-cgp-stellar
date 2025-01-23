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
    pub token_id: BytesN<32>,
    pub token_address: Address,
    pub amount: i128,
    #[data]
    pub payload: Bytes,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct TokenSentEvent {
    pub sender: Address,
    pub token_id: BytesN<32>,
    pub destination_chain: String,
    pub destination_app_contract: Bytes,
    pub amount: i128,
    #[data]
    pub recipient: Option<Bytes>,
}
