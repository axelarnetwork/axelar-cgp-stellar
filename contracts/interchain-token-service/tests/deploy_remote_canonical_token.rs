mod utils;

use axelar_soroban_std::address::AddressExt;
use axelar_soroban_std::{auth_invocation, events};
use interchain_token_service::event::InterchainTokenDeploymentStartedEvent;
use interchain_token_service::types::{
    DeployInterchainToken, HubMessage, Message, TokenManagerType,
};
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::token::{self, StellarAssetClient};
use soroban_sdk::{Address, Bytes, IntoVal, String, Symbol};
use soroban_token_sdk::metadata::TokenMetadata;
use utils::{setup_env, setup_gas_token};

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
    let expected_deploy_salt = client.canonical_token_deploy_salt(&token_address);
    let expected_id = client.interchain_token_id(&Address::zero(&env), &expected_deploy_salt);
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
    let token_metadata = TokenMetadata {
        name: token.name(),
        decimal: token.decimals(),
        symbol: token.symbol(),
    };

    let message = Message::DeployInterchainToken(DeployInterchainToken {
        token_id: expected_id.clone(),
        name: token_metadata.name.clone(),
        symbol: token_metadata.symbol.clone(),
        decimals: token_metadata.decimal as u8,
        minter: None,
    });

    let payload = HubMessage::SendToHub {
        destination_chain: destination_chain.clone(),
        message,
    }
    .abi_encode(&env);

    let deployed_token_id = client
        .mock_all_auths_allowing_non_root_auth()
        .deploy_remote_canonical_token(&token_address, &destination_chain, &spender, &gas_token);
    assert_eq!(expected_id, deployed_token_id);

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeploymentStartedEvent,
    >(&env, -4));

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
            client.address,
            its_hub_chain,
            its_hub_address,
            payload,
            spender,
            gas_token,
            Bytes::new(&env)
        ),
        transfer_auth
    );

    assert_eq!(env.auths(), gas_service_auth);
}

#[test]
#[should_panic(expected = "HostError: Error(Storage, MissingValue)")]
fn deploy_remote_canonical_token_fail_no_actual_token() {
    let (env, client, _, _, _) = setup_env();

    let spender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &spender);
    let token_address = Address::generate(&env);
    let expected_deploy_salt = client.canonical_token_deploy_salt(&token_address);
    let expected_id = client.interchain_token_id(&Address::zero(&env), &expected_deploy_salt);

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

    client
        .mock_all_auths_allowing_non_root_auth()
        .deploy_remote_canonical_token(&token_address, &destination_chain, &spender, &gas_token);
}
