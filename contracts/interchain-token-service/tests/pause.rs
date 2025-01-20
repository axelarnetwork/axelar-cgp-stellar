mod utils;

use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String};
use stellar_axelar_gateway::testutils::TestSignerSet;
use stellar_axelar_gateway::types::Message as GatewayMessage;
use stellar_axelar_gateway::AxelarGatewayClient;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::traits::BytesExt;
use stellar_axelar_std::{assert_auth, assert_auth_err, assert_contract_err, events};
use stellar_interchain_token_service::error::ContractError;
use stellar_interchain_token_service::event::{FlowLimitSetEvent, PauseStatusSetEvent};
use stellar_interchain_token_service::types::{HubMessage, InterchainTransfer, Message};
use stellar_interchain_token_service::InterchainTokenServiceClient;
use utils::{
    approve_gateway_messages, register_chains, setup_env, setup_gas_token, setup_its_token,
};

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
