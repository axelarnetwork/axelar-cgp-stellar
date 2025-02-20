use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String, Vec};
use stellar_axelar_std::{assert_auth, assert_auth_err, assert_contract_err, events};

use super::utils::setup_env;
use crate::error::ContractError;
use crate::event::{TrustedChainRemovedEvent, TrustedChainSetEvent};

fn setup_chains(env: &Env, chain_names: &[&str]) -> Vec<String> {
    let mut chains = Vec::new(env);
    for &name in chain_names {
        chains.push_back(String::from_str(env, name));
    }
    chains
}

#[test]
fn set_trusted_chain() {
    let (env, client, _, _, _) = setup_env();

    let chain = String::from_str(&env, "chain");

    assert_auth!(client.owner(), client.set_trusted_chain(&chain));

    goldie::assert!(events::fmt_last_emitted_event::<TrustedChainSetEvent>(&env));

    assert!(client.is_trusted_chain(&chain));
}

#[test]
fn set_trusted_chain_fails_if_not_owner() {
    let (env, client, _, _, _) = setup_env();

    let not_owner = Address::generate(&env);
    let chain = String::from_str(&env, "chain");

    assert_auth_err!(not_owner, client.set_trusted_chain(&chain));
}

#[test]
fn set_trusted_chain_fails_if_already_set() {
    let (env, client, _, _, _) = setup_env();

    let chain = String::from_str(&env, "chain");
    client.mock_all_auths().set_trusted_chain(&chain);

    assert_contract_err!(
        client.mock_all_auths().try_set_trusted_chain(&chain),
        ContractError::TrustedChainAlreadySet
    );
}

#[test]
fn remove_trusted_chain() {
    let (env, client, _, _, _) = setup_env();

    let chain = String::from_str(&env, "chain");

    assert_auth!(client.owner(), client.set_trusted_chain(&chain));

    assert_auth!(client.owner(), client.remove_trusted_chain(&chain));

    goldie::assert!(events::fmt_last_emitted_event::<TrustedChainRemovedEvent>(
        &env
    ));

    assert!(!client.is_trusted_chain(&chain));
}

#[test]
fn remove_trusted_chain_fails_if_not_owner() {
    let (env, client, _, _, _) = setup_env();

    let not_owner = Address::generate(&env);
    let chain = String::from_str(&env, "chain");

    assert_auth_err!(not_owner, client.remove_trusted_chain(&chain));
}

#[test]
fn remove_trusted_chain_fails_if_not_set() {
    let (env, client, _, _, _) = setup_env();

    let chain = String::from_str(&env, "chain");

    assert!(!client.is_trusted_chain(&chain));

    assert_contract_err!(
        client.mock_all_auths().try_remove_trusted_chain(&chain),
        ContractError::TrustedChainNotSet
    );
}

#[test]
fn set_trusted_chains() {
    let (env, client, _, _, _) = setup_env();

    let chains = setup_chains(&env, &["chain1", "chain2"]);

    assert_auth!(client.owner(), client.set_trusted_chains(&chains));

    for chain in &chains {
        assert!(client.is_trusted_chain(&chain));
    }
}

#[test]
fn set_trusted_chains_fails_if_not_owner() {
    let (env, client, _, _, _) = setup_env();

    let not_owner = Address::generate(&env);
    let chains = setup_chains(&env, &["chain1", "chain2"]);

    assert_auth_err!(not_owner, client.set_trusted_chains(&chains));
}

#[test]
fn set_trusted_chains_fails_if_already_set() {
    let (env, client, _, _, _) = setup_env();

    let chains = setup_chains(&env, &["chain1", "chain2"]);

    client.mock_all_auths().set_trusted_chains(&chains);

    assert_contract_err!(
        client.mock_all_auths().try_set_trusted_chains(&chains),
        ContractError::TrustedChainAlreadySet
    );
}

#[test]
fn remove_trusted_chains() {
    let (env, client, _, _, _) = setup_env();

    let chains = setup_chains(&env, &["chain1", "chain2"]);

    assert_auth!(client.owner(), client.set_trusted_chains(&chains));

    assert_auth!(client.owner(), client.remove_trusted_chains(&chains));

    for chain in &chains {
        assert!(!client.is_trusted_chain(&chain));
    }
}

#[test]
fn remove_trusted_chains_fails_if_not_set() {
    let (env, client, _, _, _) = setup_env();

    let chains = setup_chains(&env, &["chain1", "chain2"]);

    for chain in &chains {
        assert!(!client.is_trusted_chain(&chain));
    }

    assert_contract_err!(
        client.mock_all_auths().try_remove_trusted_chains(&chains),
        ContractError::TrustedChainNotSet
    );
}
