use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;
use stellar_axelar_std::interfaces::{PausedEvent, UnpausedEvent};
use stellar_axelar_std::{assert_auth, assert_auth_err, events};

use super::utils::setup_env;

#[test]
fn pause_succeeds() {
    let (env, client, _, _, _) = setup_env();

    assert!(!client.paused());

    assert_auth!(client.owner(), client.pause());
    goldie::assert!(events::fmt_last_emitted_event::<PausedEvent>(&env));

    assert!(client.paused());
}

#[test]
fn unpause_succeeds() {
    let (env, client, _, _, _) = setup_env();

    assert_auth!(client.owner(), client.pause());

    assert!(client.paused());
    assert_auth!(client.owner(), client.unpause());

    goldie::assert!(events::fmt_last_emitted_event::<UnpausedEvent>(&env));

    assert!(!client.paused());
}

#[test]
fn pause_fails_with_invalid_auth() {
    let (env, client, _, _, _) = setup_env();

    assert_auth_err!(Address::generate(&env), client.pause());
}
