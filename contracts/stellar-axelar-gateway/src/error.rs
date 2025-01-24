use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    /// Upgradable
    MigrationNotAllowed = 1,
    /// Auth
    InvalidThreshold = 2,
    InvalidProof = 3,
    InvalidSigners = 4,
    InsufficientRotationDelay = 5,
    InvalidSignatures = 6,
    InvalidWeight = 7,
    WeightOverflow = 8,
    NotLatestSigners = 9,
    DuplicateSigners = 10,
    InvalidSignersHash = 11,
    InvalidEpoch = 12,
    EmptySigners = 13,
    OutdatedSigners = 14,
    /// Messages
    EmptyMessages = 15,
    /// Pausable
    ContractPaused = 16,
}
