use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_std::{assert_auth, assert_auth_err, assert_contract_err, events};
use stellar_interchain_token::InterchainTokenClient;

use super::utils::{setup_env, TokenMetadataExt};
use crate::error::ContractError;
use crate::event::{InterchainTokenDeployedEvent, TokenManagerDeployedEvent};
use crate::tests::utils::{
    INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX, INTERCHAIN_TOKEN_DEPLOYED_NO_SUPPLY_EVENT_IDX,
    TOKEN_MANAGER_DEPLOYED_EVENT_IDX,
};
use crate::types::TokenManagerType;

fn dummy_token_params(env: &Env) -> (Address, BytesN<32>, TokenMetadata) {
    let sender = Address::generate(env);
    let salt = BytesN::<32>::from_array(env, &[1; 32]);
    let token_metadata = TokenMetadata::new(env, "Test", "TEST", 6);

    (sender, salt, token_metadata)
}

#[test]
fn deploy_interchain_token_succeeds() {
    let (env, client, _, _, _) = setup_env();

    let (sender, salt, token_metadata) = dummy_token_params(&env);
    let minter: Option<Address> = None;
    let initial_supply = 100;

    assert_auth!(
        &sender,
        client.deploy_interchain_token(&sender, &salt, &token_metadata, &initial_supply, &minter)
    );
    let interchain_token_deployed_event = events::fmt_emitted_event_at_idx::<
        InterchainTokenDeployedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX);
    let token_manager_deployed_event = events::fmt_emitted_event_at_idx::<TokenManagerDeployedEvent>(
        &env,
        TOKEN_MANAGER_DEPLOYED_EVENT_IDX,
    );

    goldie::assert!([
        interchain_token_deployed_event,
        token_manager_deployed_event
    ]
    .join("\n\n"));
}

#[test]
fn deploy_interchain_token_fails_when_paused() {
    let (env, client, _, _, _) = setup_env();

    client.mock_all_auths().pause();

    assert_contract_err!(
        client.try_deploy_interchain_token(
            &Address::generate(&env),
            &BytesN::from_array(&env, &[1; 32]),
            &TokenMetadata::new(&env, "Test", "TEST", 6),
            &1,
            &None
        ),
        ContractError::ContractPaused
    );
}

#[test]
fn deploy_interchain_token_fails_when_already_deployed() {
    let (env, client, _, _, _) = setup_env();

    let (sender, salt, token_metadata) = dummy_token_params(&env);
    let minter: Option<Address> = None;
    let initial_supply = 100;

    client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &minter,
    );

    assert_contract_err!(
        client.mock_all_auths().try_deploy_interchain_token(
            &sender,
            &salt,
            &token_metadata,
            &initial_supply,
            &minter
        ),
        ContractError::TokenAlreadyRegistered
    );
}

#[test]
fn deploy_interchain_token_with_initial_supply_no_minter() {
    let (env, client, _, _, _) = setup_env();

    let (sender, salt, token_metadata) = dummy_token_params(&env);
    let minter: Option<Address> = None;
    let initial_supply = 100;

    let token_id = assert_auth!(
        &sender,
        client.deploy_interchain_token(&sender, &salt, &token_metadata, &initial_supply, &minter)
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeployedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX));

    let token_address = client.registered_token_address(&token_id);
    let token_manager = client.deployed_token_manager(&token_id);
    let token = InterchainTokenClient::new(&env, &token_address);

    assert_eq!(token.owner(), client.address);
    assert!(!token.is_minter(&client.address));
    assert!(token.is_minter(&token_manager));
    assert!(!token.is_minter(&sender));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_with_initial_supply_valid_minter() {
    let (env, client, _, _, _) = setup_env();

    let (sender, salt, token_metadata) = dummy_token_params(&env);
    let minter = Address::generate(&env);
    let initial_supply = 100;

    let token_id = assert_auth!(
        &sender,
        client.deploy_interchain_token(
            &sender,
            &salt,
            &token_metadata,
            &initial_supply,
            &Some(minter.clone()),
        )
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeployedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX));

    let token_address = client.registered_token_address(&token_id);
    let token_manager = client.deployed_token_manager(&token_id);
    let token = InterchainTokenClient::new(&env, &token_address);

    assert_eq!(token.owner(), client.address);
    assert!(!token.is_minter(&client.address));
    assert!(token.is_minter(&token_manager));
    assert!(token.is_minter(&minter));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_check_token_id_and_token_manager_type() {
    let (env, client, _, _, _) = setup_env();

    let (sender, salt, token_metadata) = dummy_token_params(&env);
    let minter = Some(Address::generate(&env));
    let initial_supply = 100;

    let expected_token_id = client.interchain_token_id(&sender, &salt);

    let token_id = assert_auth!(
        &sender,
        client.deploy_interchain_token(&sender, &salt, &token_metadata, &initial_supply, &minter)
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeployedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX));

    assert_eq!(token_id, expected_token_id);
    assert_eq!(
        client.token_manager_type(&token_id),
        TokenManagerType::NativeInterchainToken
    );
}

#[test]
fn deploy_interchain_token_zero_initial_supply_and_valid_minter() {
    let (env, client, _, _, _) = setup_env();

    let (sender, salt, token_metadata) = dummy_token_params(&env);
    let minter = Address::generate(&env);
    let initial_supply = 0;

    let token_id = assert_auth!(
        &sender,
        client.deploy_interchain_token(
            &sender,
            &salt,
            &token_metadata,
            &initial_supply,
            &Some(minter.clone()),
        )
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeployedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_NO_SUPPLY_EVENT_IDX));

    let token_address = client.registered_token_address(&token_id);
    let token_manager = client.deployed_token_manager(&token_id);
    let token = InterchainTokenClient::new(&env, &token_address);

    assert_eq!(token.owner(), client.address);
    assert!(token.is_minter(&token_manager));
    assert!(!token.is_minter(&sender));
    assert!(token.is_minter(&minter));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_fails_with_zero_initial_supply_and_no_minter() {
    let (env, client, _, _, _) = setup_env();

    let (sender, salt, token_metadata) = dummy_token_params(&env);
    let minter: Option<Address> = None;
    let initial_supply = 0;

    let token_id = client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &minter,
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeployedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_NO_SUPPLY_EVENT_IDX));

    let token_address = client.registered_token_address(&token_id);
    let token_manager = client.deployed_token_manager(&token_id);
    let token = InterchainTokenClient::new(&env, &token_address);

    assert_eq!(token.owner(), client.address);
    assert!(token.is_minter(&token_manager));
    assert!(!token.is_minter(&sender));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_fails_with_invalid_token_metadata() {
    let (env, client, _, _, _) = setup_env();

    let sender = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let initial_supply = 1000;

    let cases = [
        (
            TokenMetadata::new(&env, "", "symbol", 6),
            ContractError::InvalidTokenName,
        ),
        (
            TokenMetadata::new(&env, "A".repeat(33).as_str(), "symbol", 6),
            ContractError::InvalidTokenName,
        ),
        (
            TokenMetadata::new(&env, "name", "", 6),
            ContractError::InvalidTokenSymbol,
        ),
        (
            TokenMetadata::new(&env, "name", "A".repeat(33).as_str(), 6),
            ContractError::InvalidTokenSymbol,
        ),
        (
            TokenMetadata::new(&env, "name", "symbol", (u8::MAX) as u32 + 1),
            ContractError::InvalidTokenDecimals,
        ),
        (
            TokenMetadata::new(&env, "name", "symbol", u32::MAX),
            ContractError::InvalidTokenDecimals,
        ),
    ];

    for (token_metadata, expected_error) in cases {
        assert_contract_err!(
            client.mock_all_auths().try_deploy_interchain_token(
                &sender,
                &salt,
                &token_metadata,
                &initial_supply,
                &minter
            ),
            expected_error
        );
    }
}

#[test]
fn deploy_interchain_token_fails_with_invalid_auth() {
    let (env, client, _, _, _) = setup_env();

    let (sender, salt, token_metadata) = dummy_token_params(&env);
    let user = Address::generate(&env);
    let minter: Option<Address> = None;

    let initial_supply = 100;

    assert_auth_err!(
        user,
        client.deploy_interchain_token(&sender, &salt, &token_metadata, &initial_supply, &minter)
    );
}

#[test]
fn deploy_interchain_token_fails_with_negative_supply() {
    let (env, client, _, _, _) = setup_env();

    let (sender, salt, token_metadata) = dummy_token_params(&env);
    let invalid_supply = -1;

    assert_contract_err!(
        client.mock_all_auths().try_deploy_interchain_token(
            &sender,
            &salt,
            &token_metadata,
            &invalid_supply,
            &None
        ),
        ContractError::InvalidInitialSupply
    );
}
