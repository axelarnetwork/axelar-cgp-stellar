mod utils;
use soroban_sdk::{testutils::Address as _, xdr::ToXdr, Address, Bytes, String};
use stellar_axelar_soroban_std::{assert_contract_err, traits::BytesExt};
use stellar_interchain_token_service::error::ContractError;
use utils::{register_chains, setup_env, setup_gas_token, setup_its_token};

#[test]
fn send_directly_to_hub_chain_fails() {
    let (env, client, _gateway_client, _, _) = setup_env();

    let sender: Address = Address::generate(&env);
    let amount = 1;
    let token_id = setup_its_token(&env, &client, &sender, amount);
    let gas_token = setup_gas_token(&env, &sender);

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &client.its_hub_chain_name(),
            &Bytes::from_hex(&env, "1234"),
            &amount,
            &None,
            &gas_token,
        ),
        ContractError::UntrustedChain
    );
}

#[test]
fn send_to_untrusted_chain_fails() {
    let (env, client, _gateway_client, _, _) = setup_env();
    register_chains(&env, &client);

    let sender: Address = Address::generate(&env);
    let amount = 1;
    let token_id = setup_its_token(&env, &client, &sender, amount);
    let gas_token = setup_gas_token(&env, &sender);

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &String::from_str(&env, "untrusted_chain"),
            &Address::generate(&env).to_xdr(&env),
            &amount,
            &None,
            &gas_token,
        ),
        ContractError::UntrustedChain
    );
}
