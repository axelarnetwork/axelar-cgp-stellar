use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, String};
use stellar_axelar_gas_service::testutils::setup_gas_token;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::assert_contract_err;
use stellar_axelar_std::traits::BytesExt;

use super::utils::setup_env;
use crate::error::ContractError;
use crate::testutils::setup_its_token;

#[test]
fn send_directly_to_hub_chain_fails() {
    let (env, client, _gateway_client, _, _) = setup_env();

    let sender: Address = Address::generate(&env);
    let amount = 1;
    let (token_id, _) = setup_its_token(&env, &client, &sender, amount);
    let gas_token = setup_gas_token(&env, &sender);

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &client.its_hub_chain_name(),
            &Bytes::from_hex(&env, "1234"),
            &amount,
            &None,
            &Some(gas_token),
        ),
        ContractError::UntrustedChain
    );
}

#[test]
fn send_to_untrusted_chain_fails() {
    let (env, client, _gateway_client, _, _) = setup_env();
    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let sender: Address = Address::generate(&env);
    let amount = 1;
    let (token_id, _) = setup_its_token(&env, &client, &sender, amount);
    let gas_token = setup_gas_token(&env, &sender);

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &String::from_str(&env, "untrusted_chain"),
            &Address::generate(&env).to_string_bytes(),
            &amount,
            &None,
            &Some(gas_token),
        ),
        ContractError::UntrustedChain
    );
}
