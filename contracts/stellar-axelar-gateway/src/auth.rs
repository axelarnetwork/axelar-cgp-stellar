use soroban_sdk::crypto::Hash;
use soroban_sdk::{Bytes, BytesN, Env, Vec};
use stellar_axelar_std::ensure;
use stellar_axelar_std::events::Event;

use crate::error::ContractError;
use crate::event::SignersRotatedEvent;
use crate::storage_types::DataKey;
use crate::types::{Proof, ProofSignature, ProofSigner, WeightedSigner, WeightedSigners};

pub fn initialize_auth(
    env: Env,
    domain_separator: BytesN<32>,
    minimum_rotation_delay: u64,
    previous_signer_retention: u64,
    initial_signers: Vec<WeightedSigners>,
) -> Result<(), ContractError> {
    env.storage().instance().set(&DataKey::Epoch, &0_u64);

    // TODO: Do we need to manually expose these in a query, or can it be read directly off of storage in Stellar?
    env.storage().instance().set(
        &DataKey::PreviousSignerRetention,
        &previous_signer_retention,
    );

    env.storage()
        .instance()
        .set(&DataKey::DomainSeparator, &domain_separator);

    env.storage()
        .instance()
        .set(&DataKey::MinimumRotationDelay, &minimum_rotation_delay);

    ensure!(!initial_signers.is_empty(), ContractError::EmptySigners);

    for signers in initial_signers.into_iter() {
        rotate_signers(&env, signers, false)?;
    }

    Ok(())
}

pub fn validate_proof(
    env: &Env,
    data_hash: &BytesN<32>,
    proof: Proof,
) -> Result<bool, ContractError> {
    let signers_set = proof.weighted_signers();

    let signers_hash = signers_set.hash(env);

    let signers_epoch = epoch_by_signers_hash(env, signers_hash.clone())?;

    let current_epoch = epoch(env);

    let is_latest_signers: bool = signers_epoch == current_epoch;

    let previous_signers_retention: u64 = env
        .storage()
        .instance()
        .get(&DataKey::PreviousSignerRetention)
        .expect("previous_signers_retention not found");

    ensure!(
        current_epoch - signers_epoch <= previous_signers_retention,
        ContractError::OutdatedSigners
    );

    let msg_hash = message_hash_to_sign(env, signers_hash, data_hash);

    ensure!(
        validate_signatures(env, msg_hash, proof),
        ContractError::InvalidSignatures
    );

    Ok(is_latest_signers)
}

pub fn rotate_signers(
    env: &Env,
    new_signers: WeightedSigners,
    enforce_rotation_delay: bool,
) -> Result<(), ContractError> {
    validate_signers(env, &new_signers)?;

    update_rotation_timestamp(env, enforce_rotation_delay)?;

    let new_signers_hash = new_signers.hash(env);

    let new_epoch: u64 = epoch(env) + 1;

    env.storage().instance().set(&DataKey::Epoch, &new_epoch);

    env.storage()
        .persistent()
        .set(&DataKey::SignersHashByEpoch(new_epoch), &new_signers_hash);

    ensure!(
        epoch_by_signers_hash(env, new_signers_hash.clone()).is_err(),
        ContractError::DuplicateSigners
    );

    env.storage().persistent().set(
        &DataKey::EpochBySignersHash(new_signers_hash.clone()),
        &new_epoch,
    );

    SignersRotatedEvent {
        epoch: new_epoch,
        signers_hash: new_signers_hash,
        signers: new_signers,
    }
    .emit(env);

    Ok(())
}

pub fn epoch(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::Epoch)
        .expect("epoch not found")
}

pub fn epoch_by_signers_hash(env: &Env, signers_hash: BytesN<32>) -> Result<u64, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::EpochBySignersHash(signers_hash))
        .ok_or(ContractError::InvalidSignersHash)
}

pub fn signers_hash_by_epoch(env: &Env, epoch: u64) -> Result<BytesN<32>, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::SignersHashByEpoch(epoch))
        .ok_or(ContractError::InvalidEpoch)
}

fn message_hash_to_sign(env: &Env, signers_hash: BytesN<32>, data_hash: &BytesN<32>) -> Hash<32> {
    let domain_separator: BytesN<32> = env
        .storage()
        .instance()
        .get(&DataKey::DomainSeparator)
        .unwrap();

    let mut msg: Bytes = domain_separator.into();
    msg.extend_from_array(&signers_hash.to_array());
    msg.extend_from_array(&data_hash.to_array());

    // TODO: use an appropriate non tx overlapping prefix
    env.crypto().keccak256(&msg)
}

fn update_rotation_timestamp(env: &Env, enforce_rotation_delay: bool) -> Result<(), ContractError> {
    let minimum_rotation_delay: u64 = env
        .storage()
        .instance()
        .get(&DataKey::MinimumRotationDelay)
        .expect("minimum_rotation_delay not found");

    let last_rotation_timestamp: u64 = env
        .storage()
        .instance()
        .get(&DataKey::LastRotationTimestamp)
        .unwrap_or(0);

    let current_timestamp = env.ledger().timestamp();

    if enforce_rotation_delay {
        ensure!(
            current_timestamp - last_rotation_timestamp >= minimum_rotation_delay,
            ContractError::InsufficientRotationDelay
        );
    }

    env.storage()
        .instance()
        .set(&DataKey::LastRotationTimestamp, &current_timestamp);

    Ok(())
}

fn validate_signatures(env: &Env, msg_hash: Hash<32>, proof: Proof) -> bool {
    let mut total_weight = 0u128;

    for ProofSigner {
        signer: WeightedSigner {
            signer: public_key,
            weight,
        },
        signature,
    } in proof.signers.iter()
    {
        if let ProofSignature::Signed(signature) = signature {
            env.crypto()
                .ed25519_verify(&public_key, msg_hash.to_bytes().as_ref(), &signature);

            total_weight = total_weight.checked_add(weight).unwrap();

            if total_weight >= proof.threshold {
                return true;
            }
        }
    }

    false
}

/// Check if signer set is valid, i.e signer/pub key hash are in sorted order,
/// weights are non-zero and sum to at least threshold
fn validate_signers(env: &Env, weighted_signers: &WeightedSigners) -> Result<(), ContractError> {
    ensure!(
        !weighted_signers.signers.is_empty(),
        ContractError::EmptySigners
    );

    // TODO: what's the min address/hash?
    let mut previous_signer = BytesN::<32>::from_array(env, &[0; 32]);
    let mut total_weight = 0u128;

    for signer in weighted_signers.signers.iter() {
        ensure!(
            previous_signer < signer.signer,
            ContractError::InvalidSigners
        );

        ensure!(signer.weight != 0, ContractError::InvalidWeight);

        previous_signer = signer.signer;
        total_weight = total_weight
            .checked_add(signer.weight)
            .ok_or(ContractError::WeightOverflow)?;
    }

    let threshold = weighted_signers.threshold;
    ensure!(
        threshold != 0 && total_weight >= threshold,
        ContractError::InvalidThreshold
    );

    Ok(())
}
