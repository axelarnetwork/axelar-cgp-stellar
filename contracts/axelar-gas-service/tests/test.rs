#![cfg(test)]
extern crate std;

use std::format;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{bytes, Address, Bytes, Env, String, Symbol};
use stellar_axelar_gas_service::error::ContractError;
use stellar_axelar_gas_service::{AxelarGasService, AxelarGasServiceClient};
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{assert_auth_err, assert_contract_err, assert_last_emitted_event};

fn setup_env<'a>() -> (Env, Address, Address, AxelarGasServiceClient<'a>) {
    let env = Env::default();

    let owner: Address = Address::generate(&env);
    let gas_collector: Address = Address::generate(&env);
    let contract_id = env.register(AxelarGasService, (&owner, &gas_collector));
    let client = AxelarGasServiceClient::new(&env, &contract_id);

    (env, contract_id, gas_collector, client)
}

fn setup_gas_token(env: &Env, gas_amount: i128) -> Token {
    let asset = env.register_stellar_asset_contract_v2(Address::generate(&env));
    Token {
        address: asset.address(),
        amount: gas_amount,
    }
}

fn mint_gas_token(env: &Env, asset: &Address, recipient: &Address, amount: &i128) {
    StellarAssetClient::new(env, asset)
        .mock_all_auths()
        .mint(recipient, amount);
}

fn message_id(env: &Env) -> String {
    String::from_str(
        env,
        &format!(
            "{}-{}",
            "0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d", 0
        ),
    )
}

fn dummy_destination_data(env: &Env) -> (String, String) {
    let destination_chain: String = String::from_str(&env, "ethereum");
    let destination_address: String =
        String::from_str(&env, "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59");

    (destination_chain, destination_address)
}

#[test]
fn register_gas_service() {
    let env = Env::default();

    let owner: Address = Address::generate(&env);
    let gas_collector = Address::generate(&env);
    let contract_id = env.register(AxelarGasService, (&owner, &gas_collector));
    let client = AxelarGasServiceClient::new(&env, &contract_id);

    assert_eq!(client.gas_collector(), gas_collector);
}

#[test]
fn pay_gas_fails_with_zero_amount() {
    let (env, _, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 0;
    let token = setup_gas_token(&env, gas_amount);

    let payload = bytes!(&env, 0x1234);
    let (destination_chain, destination_address) = dummy_destination_data(&env);

    assert_contract_err!(
        client.mock_all_auths().try_pay_gas(
            &sender,
            &destination_chain,
            &destination_address,
            &payload,
            &spender,
            &token,
            &Bytes::new(&env),
        ),
        ContractError::InvalidAmount
    );
}

#[test]
fn pay_gas_fails_with_insufficient_user_balance() {
    let (env, _, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 2;
    let token = setup_gas_token(&env, gas_amount);
    mint_gas_token(&env, &token.address, &spender, &(gas_amount - 1));

    let payload = bytes!(&env, 0x1234);
    let (destination_chain, destination_address) = dummy_destination_data(&env);

    assert!(client
        .mock_all_auths()
        .try_pay_gas(
            &sender,
            &destination_chain,
            &destination_address,
            &payload,
            &spender,
            &token,
            &Bytes::new(&env),
        )
        .is_err());
}

#[test]
fn pay_gas() {
    let (env, contract_id, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 1;
    let token = setup_gas_token(&env, gas_amount);
    let token_client = TokenClient::new(&env, &token.address);
    mint_gas_token(&env, &token.address, &spender, &gas_amount);

    let payload = bytes!(&env, 0x1234);
    let (destination_chain, destination_address) = dummy_destination_data(&env);

    client.mock_all_auths().pay_gas(
        &sender,
        &destination_chain,
        &destination_address,
        &payload,
        &spender,
        &token,
        &Bytes::new(&env),
    );

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "gas_paid"),
            sender,
            destination_chain,
            destination_address,
            env.crypto().keccak256(&payload),
            spender.clone(),
            token,
        ),
        (Bytes::new(&env),),
    );

    assert_eq!(0, token_client.balance(&spender));
    assert_eq!(gas_amount, token_client.balance(&contract_id));
}

#[test]
fn add_gas_fails_with_zero_gas_amount() {
    let (env, _, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let message_id = message_id(&env);
    let gas_amount: i128 = 0;
    let token = setup_gas_token(&env, gas_amount);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_add_gas(&sender, &message_id, &spender, &token,),
        ContractError::InvalidAmount
    );
}

#[test]
fn add_gas_fails_with_insufficient_user_balance() {
    let (env, _, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let message_id = message_id(&env);
    let gas_amount: i128 = 2;
    let token = setup_gas_token(&env, gas_amount);
    mint_gas_token(&env, &token.address, &spender, &(gas_amount - 1));

    assert!(client
        .mock_all_auths()
        .try_add_gas(&sender, &message_id, &spender, &token,)
        .is_err());
}

#[test]
fn add_gas() {
    let (env, contract_id, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 1;
    let token = setup_gas_token(&env, gas_amount);
    let token_client = TokenClient::new(&env, &token.address);
    mint_gas_token(&env, &token.address, &spender, &gas_amount);

    let message_id = message_id(&env);
    client
        .mock_all_auths()
        .add_gas(&sender, &message_id, &spender, &token);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "gas_added"),
            sender,
            message_id,
            spender.clone(),
            token,
        ),
        (),
    );

    assert_eq!(0, token_client.balance(&spender));
    assert_eq!(gas_amount, token_client.balance(&contract_id));
}

#[test]
fn collect_fees_fails_with_zero_refund_amount() {
    let (env, contract_id, gas_collector, client) = setup_env();

    let supply: i128 = 1000;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    mint_gas_token(&env, &asset.address(), &contract_id, &supply);

    let refund_amount = 0;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_collect_fees(&gas_collector, &token),
        ContractError::InvalidAmount
    );
}

#[test]
fn collect_fees_fails_with_insufficient_balance() {
    let (env, contract_id, gas_collector, client) = setup_env();

    let supply: i128 = 5;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    mint_gas_token(&env, &asset.address(), &contract_id, &supply);

    let refund_amount = 10;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_collect_fees(&gas_collector, &token),
        ContractError::InsufficientBalance
    );
}

#[test]
fn collect_fees_fails_without_authorization() {
    let (env, contract_id, _, client) = setup_env();

    let supply: i128 = 1000;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    mint_gas_token(&env, &asset.address(), &contract_id, &supply);

    let refund_amount = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    let user: Address = Address::generate(&env);

    assert_auth_err!(user, client.collect_fees(&user, &token));
}

#[test]
fn collect_fees() {
    let (env, contract_id, gas_collector, client) = setup_env();

    let supply: i128 = 1000;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    mint_gas_token(&env, &asset.address(), &contract_id, &supply);

    let token_client = TokenClient::new(&env, &asset.address());

    let refund_amount = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    client.mock_all_auths().collect_fees(&gas_collector, &token);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "gas_collected"),
            gas_collector.clone(),
            token,
        ),
        (),
    );

    assert_eq!(refund_amount, token_client.balance(&gas_collector));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));
}

#[test]
fn refund_fails_without_authorization() {
    let (env, contract_id, _, client) = setup_env();

    let supply: i128 = 1000;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    mint_gas_token(&env, &asset.address(), &contract_id, &supply);

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };
    let message_id = message_id(&env);
    let user: Address = Address::generate(&env);

    assert_auth_err!(user, client.refund(&message_id, &receiver, &token));
}

#[test]
fn refund_fails_with_insufficient_balance() {
    let (env, contract_id, _, client) = setup_env();

    let supply: i128 = 1;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    mint_gas_token(&env, &asset.address(), &contract_id, &supply);

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 2;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    let message_id = message_id(&env);

    assert!(client
        .mock_all_auths()
        .try_refund(&message_id, &receiver, &token)
        .is_err());
}

#[test]
fn refund() {
    let (env, contract_id, _, client) = setup_env();

    let supply: i128 = 1000;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    mint_gas_token(&env, &asset.address(), &contract_id, &supply);

    let token_client = TokenClient::new(&env, &asset.address());

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    let message_id = message_id(&env);

    client
        .mock_all_auths()
        .refund(&message_id, &receiver, &token);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "gas_refunded"),
            message_id,
            receiver.clone(),
            token,
        ),
        (),
    );

    assert_eq!(refund_amount, token_client.balance(&receiver));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));
}
