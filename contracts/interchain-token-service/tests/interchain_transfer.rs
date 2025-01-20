mod utils;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, String};
use stellar_axelar_std::traits::BytesExt;
use stellar_axelar_std::{assert_contract_err, events};
use stellar_interchain_token_service::error::ContractError;
use stellar_interchain_token_service::event::InterchainTransferSentEvent;
use utils::{register_chains, setup_env, setup_gas_token, setup_its_token};

#[test]
fn interchain_transfer_send_succeeds() {
    let (env, client, _, _, _) = setup_env();

    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let amount = 1000;
    let token_id = setup_its_token(&env, &client, &sender, amount);

    let destination_chain = String::from_str(&env, "ethereum");
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = Some(Bytes::from_hex(&env, "abcd"));

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &gas_token,
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTransferSentEvent,
    >(&env, -4));
}

#[test]
fn interchain_transfer_send_fails_when_paused() {
    let (env, client, _, _, _) = setup_env();

    client.mock_all_auths().set_pause_status(&true);

    assert_contract_err!(
        client.try_interchain_transfer(
            &Address::generate(&env),
            &BytesN::from_array(&env, &[0u8; 32]),
            &String::from_str(&env, ""),
            &Bytes::from_hex(&env, ""),
            &1,
            &Some(Bytes::from_hex(&env, "")),
            &setup_gas_token(&env, &Address::generate(&env))
        ),
        ContractError::ContractPaused
    );
}

#[test]
#[should_panic(expected = "burn, Error(Contract, #9)")]
fn interchain_transfer_send_fails_on_insufficient_balance() {
    let (env, client, _, _, _) = setup_env();
    register_chains(&env, &client);

    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let amount = 1000;
    let token_id = setup_its_token(&env, &client, &sender, amount);

    let destination_chain = client.its_hub_chain_name();
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = Some(Bytes::from_hex(&env, "abcd"));

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &(amount + 1),
        &data,
        &gas_token,
    );
}

#[test]
fn interchain_transfer_fails_on_zero_amount() {
    let (env, client, _, _, _) = setup_env();
    register_chains(&env, &client);

    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let amount = 0;
    let token_id = setup_its_token(&env, &client, &sender, amount);

    let destination_chain = client.its_hub_chain_name();
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = Some(Bytes::from_hex(&env, "abcd"));

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &destination_chain,
            &destination_address,
            &amount,
            &data,
            &gas_token
        ),
        ContractError::InvalidAmount
    );
}

#[test]
fn interchain_transfer_fails_on_empty_destination_address() {
    let (env, client, _, _, _) = setup_env();
    register_chains(&env, &client);

    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let amount = 1000;
    let token_id = setup_its_token(&env, &client, &sender, amount);

    let destination_chain = client.its_hub_chain_name();
    let destination_address = Bytes::new(&env);
    let data = Some(Bytes::from_hex(&env, "abcd"));

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &destination_chain,
            &destination_address,
            &amount,
            &data,
            &gas_token
        ),
        ContractError::InvalidDestinationAddress
    );
}

#[test]
fn interchain_transfer_fails_on_empty_data() {
    let (env, client, _, _, _) = setup_env();
    register_chains(&env, &client);

    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let amount = 1000;
    let token_id = setup_its_token(&env, &client, &sender, amount);

    let destination_chain = client.its_hub_chain_name();
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let empty_data = Some(Bytes::new(&env));

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &destination_chain,
            &destination_address,
            &amount,
            &empty_data,
            &gas_token
        ),
        ContractError::InvalidData
    );
}
