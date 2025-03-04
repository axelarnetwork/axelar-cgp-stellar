use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::token::{self, StellarAssetClient};
use soroban_sdk::{Address, Bytes, BytesN, IntoVal, String, Symbol};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gas_service::testutils::setup_gas_token;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{auth_invocation, events};

use super::utils::{setup_env, TokenMetadataExt};
use crate::event::InterchainTokenDeploymentStartedEvent;
use crate::tests::utils::{
    INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX, INTERCHAIN_TOKEN_DEPLOYED_WITHOUT_GAS_TOKEN_EVENT_IDX,
};
use crate::types::{DeployInterchainToken, HubMessage, Message, TokenManagerType};

#[test]
fn deploy_remote_canonical_token_succeeds() {
    let (env, client, _, gas_service, _) = setup_env();
    let spender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &spender);
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    let initial_amount = 1;

    StellarAssetClient::new(&env, &asset.address())
        .mock_all_auths()
        .mint(&spender, &initial_amount);

    let token_address = asset.address();
    let expected_id = client.canonical_interchain_token_id(&token_address);
    assert_eq!(client.register_canonical_token(&token_address), expected_id);
    assert_eq!(client.token_address(&expected_id), token_address);
    assert_eq!(
        client.token_manager_type(&expected_id),
        TokenManagerType::LockUnlock
    );

    let destination_chain = String::from_str(&env, "ethereum");
    let its_hub_chain = String::from_str(&env, "axelar");
    let its_hub_address = String::from_str(&env, "its_hub_address");

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    let token = token::Client::new(&env, &asset.address());
    let message = Message::DeployInterchainToken(DeployInterchainToken {
        token_id: expected_id.clone(),
        name: token.symbol(),
        symbol: token.symbol(),
        decimals: token.decimals() as u8,
        minter: None,
    });
    let payload = HubMessage::SendToHub {
        destination_chain: destination_chain.clone(),
        message,
    }
    .abi_encode(&env);

    let deployed_token_id = client.mock_all_auths().deploy_remote_canonical_token(
        &token_address,
        &destination_chain,
        &spender,
        &Some(gas_token.clone()),
    );
    assert_eq!(expected_id, deployed_token_id);

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeploymentStartedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX));

    let transfer_auth = auth_invocation!(
        &env,
        spender,
        gas_token.transfer(
            spender.clone(),
            gas_service.address.clone(),
            gas_token.amount
        )
    );

    let gas_service_auth = auth_invocation!(
        &env,
        spender,
        gas_service.pay_gas(
            client.address.clone(),
            its_hub_chain,
            its_hub_address,
            payload,
            spender.clone(),
            gas_token.clone(),
            Bytes::new(&env)
        ),
        transfer_auth
    );

    let deploy_remote_canonical_token_auth = auth_invocation!(
        &env,
        spender,
        client.deploy_remote_canonical_token(
            token_address,
            destination_chain,
            spender,
            Some(gas_token)
        ),
        gas_service_auth
    );

    assert_eq!(env.auths(), deploy_remote_canonical_token_auth);
}

#[test]
fn deploy_remote_canonical_token_succeeds_without_gas_token() {
    let (env, client, _, _, _) = setup_env();
    let spender = Address::generate(&env);
    let gas_token: Option<Token> = None;
    let token_address = client.native_token_address();
    let destination_chain = String::from_str(&env, "ethereum");

    client.register_canonical_token(&token_address);

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().deploy_remote_canonical_token(
        &token_address,
        &destination_chain,
        &spender,
        &gas_token,
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeploymentStartedEvent,
    >(
        &env,
        INTERCHAIN_TOKEN_DEPLOYED_WITHOUT_GAS_TOKEN_EVENT_IDX
    ));

    let deploy_remote_canonical_token_auth = auth_invocation!(
        &env,
        spender,
        client.deploy_remote_canonical_token(token_address, destination_chain, spender, gas_token)
    );

    assert_eq!(env.auths(), deploy_remote_canonical_token_auth);
}

#[test]
fn deploy_remote_canonical_token_succeeds_native_token() {
    let (env, client, _, _, _) = setup_env();
    let spender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &spender);
    let token_address = client.native_token_address();
    let destination_chain = String::from_str(&env, "ethereum");

    client.register_canonical_token(&token_address);

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);
    client.mock_all_auths().deploy_remote_canonical_token(
        &token_address,
        &destination_chain,
        &spender,
        &Some(gas_token),
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeploymentStartedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX));
}

#[test]
fn deploy_remote_canonical_token_succeeds_without_name_truncation() {
    let (env, client, _, _, _) = setup_env();
    let spender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &spender);

    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 255);
    let initial_supply = 1;
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_id = client.mock_all_auths().deploy_interchain_token(
        &Address::generate(&env),
        &salt,
        &token_metadata,
        &initial_supply,
        &minter,
    );
    let token_address = client.token_address(&token_id);
    let destination_chain = String::from_str(&env, "ethereum");

    client.register_canonical_token(&token_address);

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);
    client.mock_all_auths().deploy_remote_canonical_token(
        &token_address,
        &destination_chain,
        &spender,
        &Some(gas_token),
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeploymentStartedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX));
}

#[test]
#[should_panic(expected = "HostError: Error(Storage, MissingValue)")]
fn deploy_remote_canonical_token_fail_no_actual_token() {
    let (env, client, _, _, _) = setup_env();

    let spender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &spender);
    let token_address = Address::generate(&env);
    let expected_id = client.canonical_interchain_token_id(&token_address);

    assert_eq!(client.register_canonical_token(&token_address), expected_id);
    assert_eq!(client.token_address(&expected_id), token_address);

    assert_eq!(
        client.token_manager_type(&expected_id),
        TokenManagerType::LockUnlock
    );

    let destination_chain = String::from_str(&env, "ethereum");
    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().deploy_remote_canonical_token(
        &token_address,
        &destination_chain,
        &spender,
        &Some(gas_token),
    );
}
