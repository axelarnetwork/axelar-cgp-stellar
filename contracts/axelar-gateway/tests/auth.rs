use axelar_gateway::error::ContractError;
use axelar_gateway::testutils::{generate_proof, generate_signers_set, randint};
use axelar_gateway::types::{ProofSignature, ProofSigner, WeightedSigner, WeightedSigners};
use axelar_gateway::AxelarGateway;
use axelar_soroban_std::{assert_auth, assert_contract_err};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, Vec,
};

mod utils;
use utils::setup_env;

#[test]
#[should_panic(expected = "Error(Contract, #13)")] // ContractError::InvalidSigners
fn fail_initialization_with_empty_signer_set() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let empty_signer_set = Vec::<WeightedSigners>::new(&env);
    let domain_separator: BytesN<32> = BytesN::random(&env);
    let previous_signers_retention = randint(0, 10) as u64;
    let minimum_rotation_delay: u64 = 0;
    let initial_signers = empty_signer_set;

    env.register(
        AxelarGateway,
        (
            owner,
            operator,
            domain_separator,
            minimum_rotation_delay,
            previous_signers_retention,
            initial_signers,
        ),
    );
}

#[test]
#[should_panic(expected = "failed ED25519 verification")]
fn fail_validate_proof_invalid_signatures() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let proof_hash: BytesN<32> = BytesN::random(&env);
    let proof = generate_proof(&env, proof_hash, signers);
    let random_hash: BytesN<32> = BytesN::random(&env);

    // NOTE: panic occurs in std function, cannot handle explicitly
    client.validate_proof(&random_hash, &proof);
}

#[test]
fn fail_validate_proof_empty_signatures() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let mut proof = generate_proof(&env, msg_hash.clone(), signers);

    let mut new_signers = Vec::new(&env);
    for signer in proof.signers.iter() {
        new_signers.push_back(ProofSigner {
            signer: signer.signer,
            signature: ProofSignature::Unsigned,
        });
    }
    proof.signers = new_signers;

    assert_contract_err!(
        client.try_validate_proof(&msg_hash, &proof),
        ContractError::InvalidSignatures
    );
}

#[test]
fn fail_validate_proof_threshold_not_met() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let mut total_weight = 0u128;

    let msg_hash = BytesN::random(&env);
    let mut proof = generate_proof(&env, msg_hash.clone(), signers);

    let mut new_signers = Vec::new(&env);
    for ProofSigner { signer, signature } in proof.signers {
        total_weight += signer.weight;

        let signature = if total_weight < proof.threshold {
            signature
        } else {
            ProofSignature::Unsigned
        };

        new_signers.push_back(ProofSigner { signer, signature });
    }
    proof.signers = new_signers;

    assert_contract_err!(
        client.try_validate_proof(&msg_hash, &proof),
        ContractError::InvalidSignatures
    );
}

#[test]
fn fail_validate_proof_invalid_signer_set() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let new_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator);

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let invalid_proof = generate_proof(&env, msg_hash.clone(), new_signers);
    assert_contract_err!(
        client.try_validate_proof(&msg_hash, &invalid_proof),
        ContractError::InvalidSignersHash
    );
}

#[test]
fn fail_validate_proof_threshold_overflow() {
    let (env, mut signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let last_index = signers.signers.signers.len() - 1;

    if let Some(mut last_signer) = signers.signers.signers.get(last_index) {
        last_signer.weight = u128::MAX - 1;
        signers.signers.signers.set(last_index, last_signer);
    }

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let proof = generate_proof(&env, msg_hash.clone(), signers.clone());

    assert_contract_err!(
        client.try_validate_proof(&msg_hash, &proof),
        ContractError::InvalidSignersHash
    );
}

#[test]
fn rotate_signers_fail_empty_signers() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let empty_signers = WeightedSigners {
        signers: Vec::<WeightedSigner>::new(&env),
        threshold: 0u128,
        nonce: BytesN::random(&env),
    };

    let data_hash = empty_signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_rotate_signers(&empty_signers, &proof, &true),
        ContractError::EmptySigners
    );
}

#[test]
fn rotate_signers_fail_zero_weight() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

    let mut new_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

    let WeightedSigner { signer, .. } = new_signers.signers.signers.pop_back_unchecked();
    new_signers
        .signers
        .signers
        .push_back(WeightedSigner { signer, weight: 0 });

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::InvalidWeight
    );
}

#[test]
fn rotate_signers_fail_weight_overflow() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

    let mut new_signers = generate_signers_set(&env, randint(3, 10), BytesN::random(&env));

    let WeightedSigner { signer, .. } = new_signers.signers.signers.pop_back_unchecked();
    new_signers.signers.signers.push_back(WeightedSigner {
        signer,
        weight: u128::MAX,
    });

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::WeightOverflow
    );
}

#[test]
fn rotate_signers_fail_zero_threshold() {
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let mut new_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

    new_signers.signers.threshold = 0u128;

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::InvalidThreshold
    );
}

#[test]
fn rotate_signers_fail_low_total_weight() {
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let mut new_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

    let total_weight = new_signers
        .signers
        .signers
        .iter()
        .map(|WeightedSigner { weight, .. }| weight)
        .reduce(|acc, weight| acc + weight)
        .expect("Empty signers");

    let new_threshold = total_weight + 1;

    new_signers.signers.threshold = new_threshold;

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::InvalidThreshold
    );
}

#[test]
fn rotate_signers_fail_wrong_signer_order() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

    let min_signers = 2; // need at least 2 signers to test incorrect ordering
    let mut new_signers =
        generate_signers_set(&env, randint(min_signers, 10), BytesN::random(&env));

    let len = new_signers.signers.signers.len();

    let mut reversed_signers = Vec::new(&env);
    for i in (0..len).rev() {
        if let Some(item) = new_signers.signers.signers.get(i) {
            reversed_signers.push_back(item);
        }
    }

    new_signers.signers.signers = reversed_signers;

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::InvalidSigners
    )
}

#[test]
fn rotate_signers_fail_duplicated_signers() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

    let new_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator.clone());
    let duplicated_signers = new_signers.clone();

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash.clone(), signers);
    assert_auth!(
        client.operator(),
        client.rotate_signers(&new_signers.signers, &proof, &true)
    );

    let proof = generate_proof(&env, data_hash, new_signers);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_rotate_signers(&duplicated_signers.signers, &proof, &true),
        ContractError::DuplicateSigners
    );
}

#[test]
fn rotate_signers_panics_on_outdated_signer_set() {
    let previous_signer_retention = randint(0, 5);
    let (env, original_signers, client) = setup_env(previous_signer_retention, randint(1, 10));

    let msg_hash: BytesN<32> = BytesN::random(&env);

    for _ in 0..(previous_signer_retention + 1) {
        let new_signers = generate_signers_set(
            &env,
            randint(1, 10),
            original_signers.domain_separator.clone(),
        );
        let data_hash = new_signers.signers.signers_rotation_hash(&env);
        let proof = generate_proof(&env, data_hash, original_signers.clone());
        assert_auth!(
            client.operator(),
            client.rotate_signers(&new_signers.signers, &proof, &true)
        );
    }

    // Proof from the first signer set should fail
    let proof = generate_proof(&env, msg_hash, original_signers.clone());

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_rotate_signers(&original_signers.signers, &proof, &true),
        ContractError::OutdatedSigners
    );
}

#[test]
fn multi_rotate_signers() {
    let previous_signer_retention = randint(1, 5);
    let (env, original_signers, client) = setup_env(previous_signer_retention, randint(1, 10));

    let msg_hash: BytesN<32> = BytesN::random(&env);

    let mut previous_signers = original_signers.clone();

    for _ in 0..previous_signer_retention {
        let new_signers = generate_signers_set(
            &env,
            randint(1, 10),
            original_signers.domain_separator.clone(),
        );

        let data_hash = new_signers.signers.signers_rotation_hash(&env);
        let proof = generate_proof(&env, data_hash.clone(), original_signers.clone());
        assert_auth!(
            client.operator(),
            client.rotate_signers(&new_signers.signers, &proof, &true)
        );

        let proof = generate_proof(&env, msg_hash.clone(), new_signers.clone());
        client.validate_proof(&msg_hash, &proof);

        let proof = generate_proof(&env, msg_hash.clone(), previous_signers.clone());
        client.validate_proof(&msg_hash, &proof);

        previous_signers = new_signers;
    }

    // Proof from the first signer set should still be valid
    let proof = generate_proof(&env, msg_hash.clone(), original_signers);
    client.validate_proof(&msg_hash, &proof);
}
