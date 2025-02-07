use soroban_sdk::{Bytes, BytesN, Env, Vec};
use stellar_axelar_std::ensure;
use stellar_axelar_std::events::Event;

use crate::error::ContractError;
use crate::event::SignersRotatedEvent;
use crate::storage;
use crate::types::{Proof, ProofSignature, ProofSigner, WeightedSigner, WeightedSigners};

pub fn initialize_auth(
    env: Env,
    domain_separator: BytesN<32>,
    minimum_rotation_delay: u64,
    previous_signer_retention: u64,
    initial_signers: Vec<WeightedSigners>,
) -> Result<(), ContractError> {
    storage::set_epoch(&env, &0_u64);
    storage::set_previous_signer_retention(&env, &previous_signer_retention);
    storage::set_domain_separator(&env, &domain_separator);
    storage::set_minimum_rotation_delay(&env, &minimum_rotation_delay);

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

    let signers_epoch = storage::try_epoch_by_signers_hash(env, signers_hash.clone())
        .ok_or(ContractError::InvalidSignersHash)?;

    let current_epoch = storage::epoch(env);

    let is_latest_signers: bool = signers_epoch == current_epoch;

    ensure!(
        current_epoch - signers_epoch <= storage::previous_signer_retention(env),
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

    let new_epoch = storage::epoch(env) + 1;

    storage::set_epoch(env, &new_epoch);

    storage::set_signers_hash_by_epoch(env, new_epoch, &new_signers_hash);

    ensure!(
        storage::try_epoch_by_signers_hash(env, new_signers_hash.clone()).is_none(),
        ContractError::DuplicateSigners
    );

    storage::set_epoch_by_signers_hash(env, new_signers_hash.clone(), &new_epoch);

    SignersRotatedEvent {
        epoch: new_epoch,
        signers_hash: new_signers_hash,
        signers: new_signers,
    }
    .emit(env);

    Ok(())
}

fn message_hash_to_sign(env: &Env, signers_hash: BytesN<32>, data_hash: &BytesN<32>) -> BytesN<32> {
    let mut msg: Bytes = storage::domain_separator(env).into();
    msg.extend_from_array(&signers_hash.to_array());
    msg.extend_from_array(&data_hash.to_array());

    env.crypto().keccak256(&msg).into()
}

fn update_rotation_timestamp(env: &Env, enforce_rotation_delay: bool) -> Result<(), ContractError> {
    let current_timestamp = env.ledger().timestamp();

    if enforce_rotation_delay {
        ensure!(
            current_timestamp - storage::last_rotation_timestamp(env)
                >= storage::minimum_rotation_delay(env),
            ContractError::InsufficientRotationDelay
        );
    }

    storage::set_last_rotation_timestamp(env, &current_timestamp);

    Ok(())
}

fn validate_signatures(env: &Env, msg_hash: BytesN<32>, proof: Proof) -> bool {
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
                .ed25519_verify(&public_key, msg_hash.as_ref(), &signature);

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
