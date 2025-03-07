#![cfg(test)]
extern crate std;

use std::format;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{bytes, Address, Bytes, Env, String};
use stellar_axelar_std::events::fmt_last_emitted_event;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{assert_auth, assert_auth_err, assert_contract_err, mock_auth};

use crate::error::ContractError;
use crate::event::{GasAddedEvent, GasCollectedEvent, GasPaidEvent, GasRefundedEvent};
use crate::{AxelarGasService, AxelarGasServiceClient};

fn setup_env<'a>() -> (Env, Address, Address, AxelarGasServiceClient<'a>) {
    let env = Env::default();

    let owner: Address = Address::generate(&env);
    let operator: Address = Address::generate(&env);
    let contract_id = env.register(AxelarGasService, (&owner, &operator));
    let client = AxelarGasServiceClient::new(&env, &contract_id);

    (env, contract_id, operator, client)
}

fn setup_token<'a>(env: &'a Env, recipient: &'a Address, amount: i128) -> (Token, TokenClient<'a>) {
    let asset = env.register_stellar_asset_contract_v2(Address::generate(env));

    StellarAssetClient::new(env, &asset.address())
        .mock_all_auths()
        .mint(recipient, &amount);

    (
        Token {
            address: asset.address(),
            amount,
        },
        TokenClient::new(env, &asset.address()),
    )
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
    let destination_chain: String = String::from_str(env, "ethereum");
    let destination_address = Address::generate(env).to_string();

    (destination_chain, destination_address)
}

#[test]
fn register_gas_service() {
    let env = Env::default();

    let owner: Address = Address::generate(&env);
    let operator = Address::generate(&env);
    let contract_id = env.register(AxelarGasService, (&owner, &operator));
    let client = AxelarGasServiceClient::new(&env, &contract_id);

    assert_eq!(client.operator(), operator);
}

#[test]
fn pay_gas_fails_with_zero_amount() {
    let (env, _, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 0;
    let token = Token {
        address: Address::generate(&env),
        amount: gas_amount,
    };

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
#[should_panic(expected = "HostError: Error(Contract, #10)")] // "balance is not sufficient to spend"
fn pay_gas_fails_with_insufficient_user_balance() {
    let (env, _, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 2;
    let (_token, token_client) = setup_token(&env, &spender, gas_amount - 1);
    let token = Token {
        address: _token.address,
        amount: gas_amount,
    };

    let payload = bytes!(&env, 0x1234);
    let (destination_chain, destination_address) = dummy_destination_data(&env);

    let transfer_token_auth = mock_auth!(
        spender,
        token_client.transfer(spender, client.address, token.amount)
    );

    let pay_gas_auth = mock_auth!(
        spender,
        client.pay_gas(
            sender,
            destination_chain,
            destination_address,
            payload,
            spender,
            token,
            Bytes::new(&env)
        ),
        &[(transfer_token_auth.invoke).clone()]
    );

    client.mock_auths(&[pay_gas_auth]).pay_gas(
        &sender,
        &destination_chain,
        &destination_address,
        &payload,
        &spender,
        &token,
        &Bytes::new(&env),
    );
}

#[test]
fn pay_gas() {
    let (env, contract_id, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 1;
    let (token, token_client) = setup_token(&env, &spender, gas_amount);

    let payload = bytes!(&env, 0x1234);
    let (destination_chain, destination_address) = dummy_destination_data(&env);

    let transfer_token_auth = mock_auth!(
        spender,
        token_client.transfer(spender, client.address, token.amount)
    );

    let pay_gas_auth = mock_auth!(
        spender,
        client.pay_gas(
            sender,
            destination_chain,
            destination_address,
            payload,
            spender,
            token,
            Bytes::new(&env)
        ),
        &[(transfer_token_auth.invoke).clone()]
    );

    client.mock_auths(&[pay_gas_auth]).pay_gas(
        &sender,
        &destination_chain,
        &destination_address,
        &payload,
        &spender,
        &token,
        &Bytes::new(&env),
    );

    goldie::assert!(fmt_last_emitted_event::<GasPaidEvent>(&env));

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
    let token = Token {
        address: Address::generate(&env),
        amount: gas_amount,
    };

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_add_gas(&sender, &message_id, &spender, &token,),
        ContractError::InvalidAmount
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")] // "balance is not sufficient to spend"
fn add_gas_fails_with_insufficient_user_balance() {
    let (env, _, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let message_id = message_id(&env);
    let gas_amount: i128 = 2;
    let (_token, _) = setup_token(&env, &spender, gas_amount - 1);
    let token = Token {
        address: _token.address,
        amount: gas_amount,
    };
    client
        .mock_all_auths()
        .add_gas(&sender, &message_id, &spender, &token);
}

#[test]
fn add_gas() {
    let (env, contract_id, _, client) = setup_env();

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 1;
    let (token, _) = setup_token(&env, &spender, gas_amount);
    let token_client = TokenClient::new(&env, &token.address);

    let message_id = message_id(&env);
    client
        .mock_all_auths()
        .add_gas(&sender, &message_id, &spender, &token);

    goldie::assert!(fmt_last_emitted_event::<GasAddedEvent>(&env));

    assert_eq!(0, token_client.balance(&spender));
    assert_eq!(gas_amount, token_client.balance(&contract_id));
}

#[test]
fn collect_fees_fails_with_zero_amount() {
    let (env, _, operator, client) = setup_env();
    let spender: Address = Address::generate(&env);
    let refund_amount = 0;
    let supply: i128 = 1000;
    let (_token, _) = setup_token(&env, &spender, supply);
    let token = Token {
        address: _token.address,
        amount: refund_amount,
    };

    assert_contract_err!(
        client.mock_all_auths().try_collect_fees(&operator, &token),
        ContractError::InvalidAmount
    );
}

#[test]
fn collect_fees_fails_with_insufficient_balance() {
    let (env, contract_id, operator, client) = setup_env();

    let supply: i128 = 5;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &asset.address())
        .mock_all_auths()
        .mint(&contract_id, &supply);

    let refund_amount = 10;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    assert_contract_err!(
        client.mock_all_auths().try_collect_fees(&operator, &token),
        ContractError::InsufficientBalance
    );
}

#[test]
fn collect_fees_fails_without_authorization() {
    let (env, contract_id, _, client) = setup_env();

    let supply: i128 = 1000;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &asset.address())
        .mock_all_auths()
        .mint(&contract_id, &supply);

    let refund_amount = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    let user: Address = Address::generate(&env);

    assert_auth_err!(user, client.collect_fees(&user, &token));
}

#[test]
fn collect_fees_succeeds() {
    let (env, contract_id, operator, client) = setup_env();

    let supply: i128 = 1000;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &asset.address())
        .mock_all_auths()
        .mint(&contract_id, &supply);

    let token_client = TokenClient::new(&env, &asset.address());

    let refund_amount = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    let transfer_token_auth = mock_auth!(
        operator,
        token_client.transfer(operator, client.address, token.amount)
    );

    let collect_fees_auth = mock_auth!(
        operator,
        client.collect_fees(&operator, &token),
        &[(transfer_token_auth.invoke).clone()]
    );

    client
        .mock_auths(&[collect_fees_auth])
        .collect_fees(&operator, &token);

    goldie::assert!(fmt_last_emitted_event::<GasCollectedEvent>(&env));

    assert_eq!(refund_amount, token_client.balance(&operator));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));
}

#[test]
fn refund_fails_without_authorization() {
    let (env, contract_id, _, client) = setup_env();

    let supply: i128 = 1000;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &asset.address())
        .mock_all_auths()
        .mint(&contract_id, &supply);

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
#[should_panic(expected = "HostError: Error(Contract, #10)")] // "balance is not sufficient to spend"
fn refund_fails_with_insufficient_balance() {
    let (env, contract_id, operator, client) = setup_env();

    let supply: i128 = 1;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &asset.address())
        .mock_all_auths()
        .mint(&contract_id, &supply);

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 2;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };
    let token_client = TokenClient::new(&env, &asset.address());
    let message_id = message_id(&env);

    let transfer_token_auth = mock_auth!(
        operator,
        token_client.transfer(operator, client.address, token.amount)
    );

    let refund_auth = mock_auth!(
        operator,
        client.refund(&message_id, &receiver, &token),
        &[(transfer_token_auth.invoke).clone()]
    );

    client
        .mock_auths(&[refund_auth])
        .refund(&message_id, &receiver, &token)
}

#[test]
fn refund_succeeds() {
    let (env, contract_id, _, client) = setup_env();

    let supply: i128 = 1000;
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &asset.address())
        .mock_all_auths()
        .mint(&contract_id, &supply);

    let token_client = TokenClient::new(&env, &asset.address());

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    let message_id = message_id(&env);

    assert_auth!(
        client.operator(),
        client.refund(&message_id, &receiver, &token)
    );

    goldie::assert!(fmt_last_emitted_event::<GasRefundedEvent>(&env));

    assert_eq!(refund_amount, token_client.balance(&receiver));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));
}
