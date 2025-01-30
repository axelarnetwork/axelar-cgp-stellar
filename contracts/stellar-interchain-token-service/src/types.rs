use soroban_sdk::{contracttype, Bytes, BytesN, String};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    InterchainTransfer(InterchainTransfer),
    DeployInterchainToken(DeployInterchainToken),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterchainTransfer {
    pub token_id: BytesN<32>,
    pub source_address: Bytes,
    pub destination_address: Bytes,
    pub amount: i128,
    pub data: Option<Bytes>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeployInterchainToken {
    pub token_id: BytesN<32>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub minter: Option<Bytes>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HubMessage {
    SendToHub {
        destination_chain: String,
        message: Message,
    },
    ReceiveFromHub {
        source_chain: String,
        message: Message,
    },
}

/// The type of token manager used for the tokenId.
///
/// Only the variants supported by Stellar ITS are defined here.
/// The variant values need to match the [ITS spec](https://github.com/axelarnetwork/interchain-token-service/blob/v2.0.0/contracts/interfaces/ITokenManagerType.sol#L9).
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TokenManagerType {
    NativeInterchainToken = 0,
    // MintBurnFrom = 1,
    LockUnlock = 2,
    // LockUnlockFee = 3,
    // MintBurn = 4,
}
