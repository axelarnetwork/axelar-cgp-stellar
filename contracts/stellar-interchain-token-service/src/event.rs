use core::fmt::Debug;

use soroban_sdk::{Address, Bytes, BytesN, String};
use stellar_axelar_std::IntoEvent;

use crate::types::TokenManagerType;

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct TrustedChainSetEvent {
    pub chain: String,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct TrustedChainRemovedEvent {
    pub chain: String,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct FlowLimitSetEvent {
    pub token_id: BytesN<32>,
    /// A `None` value implies that flow limit checks have been disabled for this `token_id`
    pub flow_limit: Option<i128>,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct InterchainTokenDeployedEvent {
    pub token_id: BytesN<32>,
    pub token_address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub minter: Option<Address>,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct TokenManagerDeployedEvent {
    pub token_id: BytesN<32>,
    pub token_address: Address,
    pub token_manager: Address,
    pub token_manager_type: TokenManagerType,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
#[event_name("token_deployment_started")]
pub struct InterchainTokenDeploymentStartedEvent {
    pub token_id: BytesN<32>,
    pub token_address: Address,
    pub destination_chain: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub minter: Option<Address>,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct InterchainTransferSentEvent {
    pub token_id: BytesN<32>,
    pub source_address: Address,
    pub destination_chain: String,
    pub destination_address: Bytes,
    pub amount: i128,
    #[data]
    pub data: Option<Bytes>,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct InterchainTransferReceivedEvent {
    pub source_chain: String,
    pub token_id: BytesN<32>,
    pub source_address: Bytes,
    pub destination_address: Address,
    pub amount: i128,
    #[data]
    pub data: Option<Bytes>,
}
