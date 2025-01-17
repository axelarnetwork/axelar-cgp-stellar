use soroban_sdk::contracterror;
use stellar_axelar_gateway::executable::NotApprovedError;
use stellar_axelar_gateway::impl_not_approved_error;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    NotOwner = 2,
    TrustedChainAlreadySet = 3,
    TrustedChainNotSet = 4,
    InvalidMessageType = 5,
    InvalidPayload = 6,
    UntrustedChain = 7,
    InsufficientMessageLength = 8,
    AbiDecodeFailed = 9,
    InvalidAmount = 10,
    InvalidUtf8 = 11,
    InvalidMinter = 12,
    InvalidDestinationAddress = 13,
    InvalidHubChain = 14,
    TokenAlreadyRegistered = 15,
    InvalidTokenMetaData = 16,
    InvalidTokenId = 17,
    TokenAlreadyDeployed = 18,
    InvalidFlowLimit = 19,
    FlowLimitExceeded = 20,
    FlowAmountOverflow = 21,
    NotApproved = 22,
    InvalidDestinationChain = 23,
}

impl_not_approved_error!(ContractError);
