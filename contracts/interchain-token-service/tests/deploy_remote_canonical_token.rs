mod utils;

use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::token::{self, StellarAssetClient};
use soroban_sdk::{Address, Bytes, IntoVal, String, Symbol};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::{auth_invocation, events};
use stellar_interchain_token_service::event::InterchainTokenDeploymentStartedEvent;
use stellar_interchain_token_service::types::{
    DeployInterchainToken, HubMessage, Message, TokenManagerType,
};
use utils::{setup_env, setup_gas_token};

#[test]
fn deploy_remote_canonical_token_succeeds() {
    let (env, client, gateway, gas_service, _) = setup_env();
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

    let payload_val = payload.clone().expect("").to_val();
    let gas_token_val = gas_token.try_into_val(&env).expect("");

    let transfer_token_auth = MockAuthInvoke {
        contract: &gas_token.address,
        fn_name: "transfer",
        args: soroban_sdk::vec![
            &env,
            spender.to_val(),
            gas_service.address.to_val(),
            gas_token.amount.into_val(&env),
        ],
        sub_invokes: &[],
    };

    let pay_gas_auth = MockAuthInvoke {
        contract: &gas_service.address,
        fn_name: "pay_gas",
        args: soroban_sdk::vec![
            &env,
            client.address.to_val(),
            its_hub_chain.to_val(),
            its_hub_address.to_val(),
            payload_val,
            spender.to_val(),
            gas_token_val,
            Bytes::new(&env).into(),
        ],
        sub_invokes: &[transfer_token_auth],
    };

    let call_contract_auth = MockAuthInvoke {
        contract: &gateway.address,
        fn_name: "call_contract",
        args: soroban_sdk::vec![
            &env,
            client.address.to_val(),
            its_hub_chain.to_val(),
            its_hub_address.to_val(),
            payload_val,
        ],
        sub_invokes: &[],
    };

    env.mock_auths(&[
        MockAuth {
            address: &spender,
            invoke: &pay_gas_auth,
        },
        MockAuth {
            address: &spender,
            invoke: &call_contract_auth,
        },
    ]);

    let deployed_token_id = client.deploy_remote_canonical_token(
        &token_address,
        &destination_chain,
        &spender,
        &gas_token,
    );
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
