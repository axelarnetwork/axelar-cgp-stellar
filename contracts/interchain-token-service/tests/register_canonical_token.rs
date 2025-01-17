mod utils;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, BytesN};
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::{assert_contract_err, events};
use stellar_interchain_token_service::error::ContractError;
use stellar_interchain_token_service::event::InterchainTokenIdClaimedEvent;
use stellar_interchain_token_service::types::TokenManagerType;
use utils::setup_env;

#[test]
fn register_canonical_token_succeeds() {
    let (env, client, _, _, _) = setup_env();
    let token_address = Address::generate(&env);
    let expected_deploy_salt = client.canonical_token_deploy_salt(&token_address);
    let expected_id = client.interchain_token_id(&Address::zero(&env), &expected_deploy_salt);

    assert_eq!(client.register_canonical_token(&token_address), expected_id);

    assert_eq!(client.token_address(&expected_id), token_address);

    assert_eq!(
        client.token_manager_type(&expected_id),
        TokenManagerType::LockUnlock
    );

    goldie::assert!(events::fmt_last_emitted_event::<
        InterchainTokenIdClaimedEvent,
    >(&env));
}

#[test]
fn register_canonical_token_fails_if_already_registered() {
    let (env, client, _, _, _) = setup_env();
    let token_address = Address::generate(&env);

    client.register_canonical_token(&token_address);

    assert_contract_err!(
        client.try_register_canonical_token(&token_address),
        ContractError::TokenAlreadyRegistered
    );
}

#[test]
fn canonical_token_id_derivation() {
    let (env, client, _, _, _) = setup_env();
    let token_address = Address::generate(&env);

    let chain_name = client.chain_name();
    let chain_name_hash: BytesN<32> = env.crypto().keccak256(&(chain_name).to_xdr(&env)).into();
    let deploy_salt = client.canonical_token_deploy_salt(&token_address);

    let token_id = client.interchain_token_id(&Address::zero(&env), &deploy_salt);

    goldie::assert_json!(vec![
        hex::encode(chain_name_hash.to_array()),
        hex::encode(deploy_salt.to_array()),
        hex::encode(token_id.to_array())
    ]);
}
