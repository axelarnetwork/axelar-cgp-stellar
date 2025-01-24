use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{bytes, vec, Address, BytesN, String};
use stellar_axelar_std::{assert_auth, assert_auth_err, assert_contract_err, events};

use super::utils::setup_env;
use crate::error::ContractError;
use crate::event::{
    ContractCalledEvent, MessageApprovedEvent, MessageExecutedEvent, SignersRotatedEvent,
};
#[cfg(any(test, feature = "testutils"))]
use crate::testutils::{
    generate_proof, generate_signers_set, generate_signers_set_with_rng, generate_test_message,
    generate_test_message_with_rng, get_approve_hash, randint,
};
use crate::types::Message;

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

fn deterministic_rng() -> rand_chacha::ChaCha20Rng {
    use rand::SeedableRng;
    rand_chacha::ChaCha20Rng::seed_from_u64(42)
}

#[test]
fn call_contract() {
    let (env, _signers, client) = setup_env(1, 5);

    let user: Address = Address::generate(&env);
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN);
    let destination_address = String::from_str(&env, DESTINATION_ADDRESS);
    let payload = bytes!(&env, 0x1234);

    assert_auth!(
        user,
        client.call_contract(&user, &destination_chain, &destination_address, &payload)
    );
    goldie::assert!(events::fmt_last_emitted_event::<ContractCalledEvent>(&env));
}

#[test]
fn validate_message() {
    let (env, _signers, client) = setup_env(1, 5);

    let (
        Message {
            source_chain,
            message_id,
            source_address,
            contract_address,
            payload_hash,
        },
        _,
    ) = generate_test_message(&env);

    let approved = assert_auth!(
        contract_address,
        client.validate_message(
            &contract_address,
            &source_chain,
            &message_id,
            &source_address,
            &payload_hash,
        )
    );
    assert!(!approved);
    assert_eq!(env.events().all().len(), 0);
}

#[test]
fn approve_message() {
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let (message, _) = generate_test_message_with_rng(&env, deterministic_rng());
    let Message {
        source_chain,
        message_id,
        source_address,
        contract_address,
        payload_hash,
    } = message.clone();

    let messages = vec![&env, message];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    client.approve_messages(&messages, &proof);
    goldie::assert!(events::fmt_last_emitted_event::<MessageApprovedEvent>(&env));

    let is_approved = client.is_message_approved(
        &source_chain,
        &message_id,
        &source_address,
        &contract_address,
        &payload_hash,
    );
    assert!(is_approved);
}

#[test]
fn execute_approved_message() {
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let (message, _) = generate_test_message_with_rng(&env, deterministic_rng());
    let Message {
        source_chain,
        message_id,
        source_address,
        contract_address,
        payload_hash,
    } = message.clone();

    let messages = vec![&env, message];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);
    client.approve_messages(&messages, &proof);

    let approved = assert_auth!(
        contract_address,
        client.validate_message(
            &contract_address,
            &source_chain,
            &message_id,
            &source_address,
            &payload_hash,
        )
    );
    assert!(approved);
    goldie::assert!(events::fmt_last_emitted_event::<MessageExecutedEvent>(&env));

    let is_approved = client.is_message_approved(
        &source_chain,
        &message_id,
        &source_address,
        &contract_address,
        &payload_hash,
    );
    assert!(!is_approved);

    let is_executed = client.is_message_executed(&source_chain, &message_id);
    assert!(is_executed);
}

#[test]
fn fail_execute_invalid_proof() {
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let (message, _) = generate_test_message(&env);

    let invalid_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator);

    let messages = vec![&env, message];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, invalid_signers);

    assert_contract_err!(
        client.try_approve_messages(&messages, &proof),
        ContractError::InvalidSignersHash
    );
}

#[test]
fn approve_messages_fail_empty_messages() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

    let messages = soroban_sdk::Vec::new(&env);
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client.try_approve_messages(&messages, &proof),
        ContractError::EmptyMessages
    );
}

#[test]
fn approve_messages_fails_when_contract_is_paused() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

    assert_auth!(client.owner(), client.pause());

    let (message, _) = generate_test_message(&env);
    let messages = vec![&env, message];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client.try_approve_messages(&messages, &proof),
        ContractError::ContractPaused
    );
}

#[test]
fn approve_messages_skip_duplicate_message() {
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let (message, _) = generate_test_message(&env);

    let messages = vec![&env, message];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);
    client.approve_messages(&messages, &proof);

    client.approve_messages(&messages, &proof);
    assert_eq!(env.events().all().len(), 0);
}

#[test]
fn rotate_signers() {
    let (env, signers, client) = setup_env(1, 5);

    let new_signers = generate_signers_set_with_rng(
        &env,
        5,
        signers.domain_separator.clone(),
        deterministic_rng(),
    );
    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);
    let bypass_rotation_delay = false;

    client.rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay);
    goldie::assert!(events::fmt_last_emitted_event::<SignersRotatedEvent>(&env));
}

#[test]
fn approve_messages_after_rotation() {
    let (env, signers, client) = setup_env(1, 5);

    let new_signers = generate_signers_set_with_rng(
        &env,
        5,
        signers.domain_separator.clone(),
        deterministic_rng(),
    );
    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);
    let bypass_rotation_delay = false;

    client.rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay);

    let (message, _) = generate_test_message_with_rng(&env, deterministic_rng());
    let messages = vec![&env, message];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, new_signers);

    client.approve_messages(&messages, &proof);
    goldie::assert!(events::fmt_last_emitted_event::<MessageApprovedEvent>(&env));
}

#[test]
fn rotate_signers_bypass_rotation_delay() {
    let (env, signers, client) = setup_env(1, 5);
    let new_signers = generate_signers_set_with_rng(
        &env,
        5,
        signers.domain_separator.clone(),
        deterministic_rng(),
    );
    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);
    let bypass_rotation_delay = true;

    assert_auth!(
        client.operator(),
        client.rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay)
    );
    goldie::assert!(events::fmt_last_emitted_event::<SignersRotatedEvent>(&env));
}

#[test]
fn rotate_signers_bypass_rotation_delay_unauthorized() {
    let (env, signers, client) = setup_env(1, 5);

    let new_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);
    let bypass_rotation_delay = true;

    assert_auth_err!(
        client.owner(),
        client.rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay)
    );

    let not_operator = Address::generate(&env);
    assert_auth_err!(
        not_operator,
        client.rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay)
    );
}

#[test]
fn rotate_signers_fail_not_latest_signers() {
    let (env, signers, client) = setup_env(1, 5);

    let bypass_rotation_delay = false;

    let first_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = first_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers.clone());
    client.rotate_signers(&first_signers.signers, &proof, &bypass_rotation_delay);

    let second_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = second_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client.try_rotate_signers(&second_signers.signers, &proof, &bypass_rotation_delay),
        ContractError::NotLatestSigners
    );
}

#[test]
fn transfer_operatorship_unauthorized() {
    let (env, _, client) = setup_env(1, randint(1, 10));
    let not_operator = Address::generate(&env);

    assert_auth_err!(
        client.owner(),
        client.transfer_operatorship(&client.owner())
    );
    assert_auth_err!(not_operator, client.transfer_operatorship(&not_operator));
}

#[test]
fn transfer_ownership_unauthorized() {
    let (env, _, client) = setup_env(1, randint(1, 10));

    let new_owner = Address::generate(&env);

    assert_auth_err!(new_owner, client.transfer_ownership(&new_owner));
    assert_auth_err!(
        client.operator(),
        client.transfer_ownership(&client.operator())
    );
}

#[test]
fn epoch_by_signers_hash() {
    let (env, signers, client) = setup_env(1, 5);

    let bypass_rotation_delay = false;

    let first_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = first_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    client.rotate_signers(&first_signers.signers, &proof, &bypass_rotation_delay);

    assert_eq!(
        client.epoch_by_signers_hash(&first_signers.signers.hash(&env)),
        client.epoch()
    );
}

#[test]
fn epoch_by_signers_hash_fail_invalid_signers() {
    let (env, _, client) = setup_env(1, 5);
    let signers_hash = BytesN::<32>::from_array(&env, &[1; 32]);

    assert_contract_err!(
        client.try_epoch_by_signers_hash(&signers_hash),
        ContractError::InvalidSignersHash
    );
}

#[test]
fn signers_hash_by_epoch() {
    let (env, signers, client) = setup_env(1, 5);

    let bypass_rotation_delay = false;

    let first_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = first_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    client.rotate_signers(&first_signers.signers, &proof, &bypass_rotation_delay);
    let epoch = client.epoch();

    assert_eq!(
        client.signers_hash_by_epoch(&epoch),
        first_signers.signers.hash(&env)
    );
}

#[test]
fn signers_hash_by_epoch_fail_invalid_epoch() {
    let (_, _, client) = setup_env(1, 5);
    let invalid_epoch = 43u64;

    assert_contract_err!(
        client.try_signers_hash_by_epoch(&invalid_epoch),
        ContractError::InvalidEpoch
    );
}

#[test]
fn version() {
    let (env, _signers, client) = setup_env(1, randint(1, 10));

    assert_eq!(
        client.version(),
        String::from_str(&env, env!("CARGO_PKG_VERSION"))
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Storage, MissingValue)")]
fn upgrade_invalid_wasm_hash() {
    let (env, _, client) = setup_env(1, randint(1, 10));

    let new_wasm_hash = BytesN::<32>::from_array(&env, &[0; 32]);
    client.mock_all_auths().upgrade(&new_wasm_hash);
}

#[test]
fn upgrade_unauthorized() {
    let (env, _signers, client) = setup_env(1, randint(1, 10));

    let not_owner = Address::generate(&env);
    let new_wasm_hash = BytesN::<32>::from_array(&env, &[0; 32]);

    assert_auth_err!(not_owner, client.upgrade(&new_wasm_hash));
    assert_auth_err!(client.operator(), client.upgrade(&new_wasm_hash));
}
