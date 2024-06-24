use soroban_sdk::{contractclient, Address, BytesN, Env, Vec, U256};

use crate::types::{Proof, WeightedSigners};

/// Interface for the Axelar Auth Verifier.
#[contractclient(name = "AxelarAuthVerifierClient")]
pub trait AxelarAuthVerifierInterface {
    fn initialize(
        env: Env,
        owner: Address,
        previous_signer_retention: u64,
        domain_separator: BytesN<32>,
        minimum_rotation_delay: u64,
        signers: Vec<BytesN<32>>,
        weights: Vec<U256>,
        threshold: U256,
        nonce: BytesN<32>,
    );

    fn epoch(env: Env) -> u64;

    fn validate_proof(env: Env, data_hash: BytesN<32>, proof: Proof) -> bool;

    fn rotate_signers(env: Env, new_signers: WeightedSigners, enforce_rotation_delay: bool);
}
