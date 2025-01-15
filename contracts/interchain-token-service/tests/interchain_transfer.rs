mod utils;

use soroban_sdk::{testutils::Address as _, Address, Bytes, String};
use stellar_axelar_std::events;
use stellar_axelar_std::traits::BytesExt;
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
