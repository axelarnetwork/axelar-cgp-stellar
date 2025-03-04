use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{Address, Bytes, BytesN, IntoVal, String, Symbol};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gas_service::testutils::setup_gas_token;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{assert_contract_err, auth_invocation, events};

use super::utils::{setup_env, TokenMetadataExt};
use crate::error::ContractError;
use crate::event::InterchainTokenDeploymentStartedEvent;
use crate::tests::utils::{
    INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX, INTERCHAIN_TOKEN_DEPLOYED_WITHOUT_GAS_TOKEN_EVENT_IDX,
};
use crate::types::{DeployInterchainToken, HubMessage, Message};

#[test]
fn deploy_remote_interchain_token_succeeds() {
    let (env, client, _, gas_service, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 1;

    let token_id = client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &minter,
    );

    let destination_chain = String::from_str(&env, "ethereum");
    let its_hub_chain = String::from_str(&env, "axelar");
    let its_hub_address = String::from_str(&env, "its_hub_address");

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    let deployed_token_id = client.mock_all_auths().deploy_remote_interchain_token(
        &sender,
        &salt,
        &destination_chain,
        &Some(gas_token.clone()),
    );

    assert_eq!(token_id, deployed_token_id);

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeploymentStartedEvent,
    >(&env, INTERCHAIN_TOKEN_DEPLOYED_EVENT_IDX));

    let message = Message::DeployInterchainToken(DeployInterchainToken {
        token_id,
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

    let transfer_auth = auth_invocation!(
        &env,
        sender,
        gas_token.transfer(&sender, gas_service.address.clone(), gas_token.amount)
    );

    let pay_gas_auth = auth_invocation!(
        &env,
        sender,
        gas_service.pay_gas(
            client.address.clone(),
            its_hub_chain,
            its_hub_address,
            payload,
            &sender,
            gas_token.clone(),
            &Bytes::new(&env)
        ),
        transfer_auth
    );

    let deploy_remote_interchain_token_auth = auth_invocation!(
        &env,
        sender,
        client.deploy_remote_interchain_token(&sender, salt, destination_chain, Some(gas_token)),
        pay_gas_auth
    );

    assert_eq!(env.auths(), deploy_remote_interchain_token_auth);
}

#[test]
fn deploy_remote_interchain_token_succeeds_without_gas_token() {
    let (env, client, _, _, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token: Option<Token> = None;
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 1;

    let token_id = client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &minter,
    );

    let destination_chain = String::from_str(&env, "ethereum");

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    let deployed_token_id = client.mock_all_auths().deploy_remote_interchain_token(
        &sender,
        &salt,
        &destination_chain,
        &gas_token,
    );

    assert_eq!(token_id, deployed_token_id);

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeploymentStartedEvent,
    >(
        &env,
        INTERCHAIN_TOKEN_DEPLOYED_WITHOUT_GAS_TOKEN_EVENT_IDX
    ));

    let deploy_remote_interchain_token_auth = auth_invocation!(
        &env,
        sender,
        client.deploy_remote_interchain_token(&sender, salt, destination_chain, gas_token)
    );

    assert_eq!(env.auths(), deploy_remote_interchain_token_auth);
}

#[test]
fn deploy_remote_interchain_token_fails_when_paused() {
    let (env, client, _, _, _) = setup_env();

    client.mock_all_auths().pause();

    assert_contract_err!(
        client.try_deploy_remote_interchain_token(
            &Address::generate(&env),
            &BytesN::from_array(&env, &[1; 32]),
            &String::from_str(&env, "ethereum"),
            &Some(setup_gas_token(&env, &Address::generate(&env)))
        ),
        ContractError::ContractPaused
    );
}

#[test]
fn deploy_remote_interchain_token_fails_untrusted_chain() {
    let (env, client, _, _, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 1;

    client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &minter,
    );

    let destination_chain = String::from_str(&env, "ethereum");

    assert_contract_err!(
        client.mock_all_auths().try_deploy_remote_interchain_token(
            &sender,
            &salt,
            &destination_chain,
            &Some(gas_token),
        ),
        ContractError::UntrustedChain
    );
}

#[test]
fn deploy_remote_interchain_token_fails_with_invalid_token_id() {
    let (env, client, _, _, _) = setup_env();

    let spender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &spender);
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);

    let destination_chain = String::from_str(&env, "ethereum");

    assert_contract_err!(
        client.mock_all_auths().try_deploy_remote_interchain_token(
            &spender,
            &salt,
            &destination_chain,
            &Some(gas_token)
        ),
        ContractError::InvalidTokenId
    );
}

#[test]
fn deploy_remote_token_fails_local_deployment() {
    let (env, client, _, _, _) = setup_env();

    let spender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &spender);
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let destination_chain = client.chain_name();

    assert_contract_err!(
        client.mock_all_auths().try_deploy_remote_interchain_token(
            &spender,
            &salt,
            &destination_chain,
            &Some(gas_token)
        ),
        ContractError::InvalidDestinationChain
    );
}
