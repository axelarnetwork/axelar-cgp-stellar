use soroban_sdk::{contractclient, BytesN, Env, Vec};
use stellar_axelar_std::interfaces::{OperatableInterface, OwnableInterface, UpgradableInterface};

use crate::error::ContractError;
use crate::types::{Message, Proof, WeightedSigners};
use crate::AxelarGatewayMessagingInterface;

#[contractclient(name = "AxelarGatewayClient")]
pub trait AxelarGatewayInterface:
    AxelarGatewayMessagingInterface + UpgradableInterface + OwnableInterface + OperatableInterface
{
    /// Returns the domain separator.
    fn domain_separator(env: &Env) -> BytesN<32>;

    /// Returns the number of epochs that previous signers are retained for after rotations.
    fn previous_signers_retention(env: &Env) -> u64;

    /// Returns the minimum delay between rotations.
    fn minimum_rotation_delay(env: &Env) -> u64;

    /// Approves a collection of messages with the provided proof.
    ///
    /// This function allows the approval of multiple messages using a cryptographic proof.
    /// It ensures that the messages are not empty and prevents replay attacks by checking
    /// if the messages have already been approved or executed.
    ///
    /// # Arguments
    /// * `messages` - A vector of messages to be approved.
    /// * `proof` - The cryptographic proof used to validate the approval.
    ///
    /// # Errors
    /// - [`ContractError::EmptyMessages`]: If the provided messages vector is empty.
    /// - Any error from `auth::validate_proof` due to an invalid proof.
    fn approve_messages(
        env: &Env,
        messages: Vec<Message>,
        proof: Proof,
    ) -> Result<(), ContractError>;

    /// Rotates to `signers` if the `proof` is valid.
    ///
    /// If `bypass_rotation_delay` is set to true, the `operator` must authorize the rotation.
    ///
    /// # Arguments
    /// * `signers` - The new set of weighted signers to be rotated in.
    /// * `proof` - The cryptographic proof used to validate the rotation.
    /// * `bypass_rotation_delay` - A boolean indicating whether to bypass the rotation delay.
    ///
    /// # Errors
    /// - [`ContractError::NotLatestSigners`]: If the provided signers are not the latest and `bypass_rotation_delay` is false.
    /// - Any error from `auth::validate_proof` due to invalid proof.
    ///
    /// # Authorization
    /// - The `operator` must authorize if `bypass_rotation_delay` is true.
    fn rotate_signers(
        env: &Env,
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

    /// Validate the `proof` for `data_hash` created by the signers.
    /// Returns a boolean indicating if the proof was created by the latest signers.
    fn validate_proof(
        env: &Env,
        data_hash: BytesN<32>,
        proof: Proof,
    ) -> Result<bool, ContractError>;
}
