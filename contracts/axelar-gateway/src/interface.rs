use crate::{
    error::ContractError,
    types::{Message, Proof, WeightedSigners},
    AxelarGatewayMessagingInterface,
};
use soroban_sdk::{contractclient, BytesN, Env, Vec};
use stellar_axelar_std::interfaces::{OperatableInterface, OwnableInterface, UpgradableInterface};

#[contractclient(name = "AxelarGatewayClient")]
pub trait AxelarGatewayInterface:
    AxelarGatewayMessagingInterface + UpgradableInterface + OwnableInterface + OperatableInterface
{
    /// Approves a collection of messages.
    fn approve_messages(
        env: Env,
        messages: Vec<Message>,
        proof: Proof,
    ) -> Result<(), ContractError>;

    // TODO: add docstring about how bypass_rotation_delay supposed to be used.
    fn rotate_signers(
        env: Env,
        signers: WeightedSigners,
        proof: Proof,
        bypass_rotation_delay: bool,
    ) -> Result<(), ContractError>;

    /// Returns the epoch of the gateway.
    fn epoch(env: &Env) -> u64;

    /// Returns the epoch by signers hash.
    fn epoch_by_signers_hash(env: &Env, signers_hash: BytesN<32>) -> Result<u64, ContractError>;

    /// Returns the signers hash by epoch.
    fn signers_hash_by_epoch(env: &Env, epoch: u64) -> Result<BytesN<32>, ContractError>;

    /// Validate the `proof` for `data_hash` created by the signers. Returns a boolean indicating if the proof was created by the latest signers.
    fn validate_proof(
        env: &Env,
        data_hash: BytesN<32>,
        proof: Proof,
    ) -> Result<bool, ContractError>;
}
