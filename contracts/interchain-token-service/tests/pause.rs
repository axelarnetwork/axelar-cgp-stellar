use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, IntoVal};
use stellar_axelar_std::{assert_auth, assert_auth_err, events};
use stellar_interchain_token_service::event::PauseStatusSetEvent;
use stellar_interchain_token_service::testutils::setup_env;

#[test]
fn pause_succeeds() {
    let (env, client, _, _, _) = setup_env();

    assert!(!client.is_paused());

    assert_auth!(client.owner(), client.set_pause_status(&true));
    goldie::assert!(events::fmt_last_emitted_event::<PauseStatusSetEvent>(&env));

    assert!(client.is_paused());
}

#[test]
fn unpause_succeeds() {
    let (env, client, _, _, _) = setup_env();

    assert_auth!(client.owner(), client.set_pause_status(&true));

    assert!(client.is_paused());
    assert_auth!(client.owner(), client.set_pause_status(&false));

    goldie::assert!(events::fmt_last_emitted_event::<PauseStatusSetEvent>(&env));

    assert!(!client.is_paused());
}

#[test]
fn pause_fails_with_invalid_auth() {
    let (env, client, _, _, _) = setup_env();

    let user = Address::generate(&env);
    assert_auth_err!(user, client.set_pause_status(&true));
}
