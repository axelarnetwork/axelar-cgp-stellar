use soroban_sdk::{contracttype, BytesN, String};

#[contracttype]
#[derive(Clone, Debug)]
pub struct MessageApprovalKey {
    pub source_chain: String,
    pub message_id: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageApprovalValue {
    NotApproved,
    Approved(BytesN<32>),
    Executed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    /// Gateway
    MessageApproval(MessageApprovalKey),
    /// Auth Module
    PreviousSignerRetention,
    DomainSeparator,
    MinimumRotationDelay,
    Epoch,
    LastRotationTimestamp,
    SignersHashByEpoch(u64),
    EpochBySignersHash(BytesN<32>),
}
