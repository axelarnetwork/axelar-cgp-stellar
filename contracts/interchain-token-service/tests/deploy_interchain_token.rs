mod utils;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::{assert_auth_err, assert_contract_err, events};
use stellar_interchain_token::InterchainTokenClient;
use stellar_interchain_token_service::error::ContractError;
use stellar_interchain_token_service::event::InterchainTokenDeployedEvent;
use stellar_interchain_token_service::types::TokenManagerType;
use utils::{setup_env, TokenMetadataExt};

#[test]
fn deploy_interchain_token_succeeds() {
    let (env, client, _, _, _) = setup_env();

    let sender = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 100;

    client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &minter,
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeployedEvent,
    >(&env, -2));
}

#[test]
fn deploy_interchain_token_with_initial_supply_no_minter() {
    let (env, client, _, _, _) = setup_env();

    let sender = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 100;

    let token_id = client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &minter,
    );
    let token_address = client.token_address(&token_id);
    let token = InterchainTokenClient::new(&env, &token_address);

    assert_eq!(token.owner(), client.address);
    assert!(token.is_minter(&client.address));
    assert!(!token.is_minter(&sender));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_with_initial_supply_valid_minter() {
    let (env, client, _, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter = Address::generate(&env);
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 100;

    let token_id = client.deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &Some(minter.clone()),
    );

    let token_address = client.token_address(&token_id);
    let token = InterchainTokenClient::new(&env, &token_address);

    assert_eq!(token.owner(), client.address);
    assert!(!token.is_minter(&client.address));
    assert!(token.is_minter(&minter));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_check_token_id_and_token_manager_type() {
    let (env, client, _, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter = Address::generate(&env);
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 100;

    let deploy_salt = client.interchain_token_deploy_salt(&sender, &salt);
    let expected_token_id = client.interchain_token_id(&Address::zero(&env), &deploy_salt);

    let token_id = client.deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &Some(minter),
    );

    assert_eq!(token_id, expected_token_id);
    assert_eq!(
        client.token_manager_type(&token_id),
        TokenManagerType::NativeInterchainToken
    );
}

#[test]
fn deploy_interchain_token_zero_initial_supply_and_valid_minter() {
    let (env, client, _, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter = Address::generate(&env);
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 0;

    let token_id = client.deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &Some(minter.clone()),
    );

    let token_address = client.token_address(&token_id);
    let token = InterchainTokenClient::new(&env, &token_address);

    assert_eq!(token.owner(), client.address);
    assert!(token.is_minter(&client.address));
    assert!(!token.is_minter(&sender));
    assert!(token.is_minter(&minter));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_falis_zero_initial_supply_and_invalid_minter() {
    let (env, client, _, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter: Option<Address> = Some(client.address.clone());
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 0;

    assert_contract_err!(
        client.try_deploy_interchain_token(
            &sender,
            &salt,
            &token_metadata,
            &initial_supply,
            &minter
        ),
        ContractError::InvalidMinter
    );
}

#[test]
fn deploy_interchain_token_zero_initial_supply_no_minter() {
    let (env, client, _, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 0;

    let token_id =
        client.deploy_interchain_token(&sender, &salt, &token_metadata, &initial_supply, &minter);

    let token_address = client.token_address(&token_id);
    let token = InterchainTokenClient::new(&env, &token_address);

    assert_eq!(token.owner(), client.address);
    assert!(token.is_minter(&client.address));
    assert!(!token.is_minter(&sender));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_fails_with_invalid_token_metadata() {
    let (env, client, _, _, _) = setup_env();

    let sender = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let initial_supply = 0;

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

    let sender = Address::generate(&env);
    let user = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);

    let initial_supply = 100;

    assert_auth_err!(
        user,
        client.deploy_interchain_token(&sender, &salt, &token_metadata, &initial_supply, &minter)
    );
}
