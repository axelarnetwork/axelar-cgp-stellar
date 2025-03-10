use soroban_sdk::{contracttype, BytesN, String};
use stellar_axelar_std::contractstorage;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageApprovalValue {
    NotApproved,
    Approved(BytesN<32>),
    Executed,
}

#[contractstorage]
enum DataKey {
    #[persistent]
    #[value(MessageApprovalValue)]
    MessageApproval {
        source_chain: String,
        message_id: String,
    },

    #[instance]
    #[value(u64)]
    PreviousSignerRetention,

    #[instance]
    #[value(BytesN<32>)]
    DomainSeparator,

    #[instance]
    #[value(u64)]
    MinimumRotationDelay,

    #[instance]
    #[value(u64)]
    Epoch,

    #[instance]
    #[value(u64)]
    LastRotationTimestamp,

    #[persistent]
    #[value(BytesN<32>)]
    SignersHashByEpoch { epoch: u64 },

    #[persistent]
    #[value(u64)]
    EpochBySignersHash { signers_hash: BytesN<32> },
}
