use std::vec;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, BytesN};
use stellar_axelar_std::{assert_contract_err, events};

use super::utils::setup_env;
use crate::error::ContractError;
use crate::event::TokenManagerDeployedEvent;
use crate::types::TokenManagerType;

#[test]
fn register_canonical_token_succeeds() {
    let (env, client, _, _, _) = setup_env();
    let owner = Address::generate(&env);
    let token = &env.register_stellar_asset_contract_v2(owner);
    let expected_id = client.canonical_interchain_token_id(&token.address());

    assert_eq!(
        client
            .mock_all_auths()
            .register_canonical_token(&token.address()),
        expected_id
    );
    let token_manager_deployed_event =
        events::fmt_emitted_event_at_idx::<TokenManagerDeployedEvent>(&env, -1);

    assert_eq!(
        client.registered_token_address(&expected_id),
        token.address()
    );
    assert_eq!(
        client.token_manager_type(&expected_id),
        TokenManagerType::LockUnlock
    );
    goldie::assert!([token_manager_deployed_event].join("\n\n"));
}

#[test]
fn register_canonical_token_fails_when_paused() {
    let (env, client, _, _, _) = setup_env();

    client.mock_all_auths().pause();

    assert_contract_err!(
        client.try_register_canonical_token(&Address::generate(&env)),
        ContractError::ContractPaused
    );
}

#[test]
fn register_canonical_token_fails_if_already_registered() {
    let (env, client, _, _, _) = setup_env();
    let owner = Address::generate(&env);
    let token = &env.register_stellar_asset_contract_v2(owner);

    client.register_canonical_token(&token.address());

    assert_contract_err!(
        client.try_register_canonical_token(&token.address()),
        ContractError::TokenAlreadyRegistered
    );
}

#[test]
fn canonical_token_id_derivation() {
    let (env, client, _, _, _) = setup_env();
    let token_address = Address::generate(&env);

    let chain_name = client.chain_name();
    let chain_name_hash: BytesN<32> = env.crypto().keccak256(&(chain_name).to_xdr(&env)).into();
    let token_id = client.canonical_interchain_token_id(&token_address);

    goldie::assert_json!(vec![
        hex::encode(chain_name_hash.to_array()),
        hex::encode(token_id.to_array())
    ]);
}

#[test]
fn register_canonical_token_fails_if_invalid_token_address() {
    let (env, client, _, _, _) = setup_env();
    let token_address = Address::generate(&env);

    assert_contract_err!(
        client.try_register_canonical_token(&token_address),
        ContractError::InvalidTokenAddress
    );
}
