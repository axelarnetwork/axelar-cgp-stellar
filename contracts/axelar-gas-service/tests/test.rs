#![cfg(test)]
extern crate std;

use std::format;

use stellar_axelar_soroban_std::{
    assert_contract_err, assert_invoke_auth_err, assert_last_emitted_event, types::Token,
};
use soroban_sdk::Bytes;
use soroban_sdk::{
    bytes,
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Address, Env, String, Symbol,
};
use stellar_axelar_gas_service::error::ContractError;
use stellar_axelar_gas_service::{AxelarGasService, AxelarGasServiceClient};

fn setup_env<'a>() -> (Env, Address, Address, AxelarGasServiceClient<'a>) {
    let env = Env::default();

    env.mock_all_auths();

    let owner: Address = Address::generate(&env);
    let gas_collector: Address = Address::generate(&env);
    let contract_id = env.register(AxelarGasService, (&owner, &gas_collector));
    let client = AxelarGasServiceClient::new(&env, &contract_id);

    (env, contract_id, gas_collector, client)
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
fn fail_pay_gas_zero_amount() {
    let (env, _, _, client) = setup_env();

    let asset = env.register_stellar_asset_contract_v2(Address::generate(&env));

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 0;
    let token = Token {
        address: asset.address(),
        amount: gas_amount,
    };
    let payload = bytes!(&env, 0x1234);
    let destination_chain: String = String::from_str(&env, "ethereum");
    let destination_address: String =
        String::from_str(&env, "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59");

    assert_contract_err!(
        client.try_pay_gas(
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
fn fail_pay_gas_not_enough_user_balance() {
    let (env, _, _, client) = setup_env();

    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 2;
    let token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    let payload = bytes!(&env, 0x1234);
    let destination_chain: String = String::from_str(&env, "ethereum");
    let destination_address: String =
        String::from_str(&env, "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59");

    StellarAssetClient::new(&env, &asset.address()).mint(&spender, &(gas_amount - 1));

    assert!(client
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

    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));

    let spender: Address = Address::generate(&env);
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 1;
    let token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    let payload = bytes!(&env, 0x1234);
    let destination_chain: String = String::from_str(&env, "ethereum");
    let destination_address: String =
        String::from_str(&env, "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59");

    let token_client = TokenClient::new(&env, &asset.address());
    StellarAssetClient::new(&env, &asset.address()).mint(&spender, &gas_amount);

    client.pay_gas(
        &sender,
        &destination_chain,
        &destination_address,
        &payload,
        &spender,
        &token,
        &Bytes::new(&env),
    );

    assert_eq!(0, token_client.balance(&spender));
    assert_eq!(gas_amount, token_client.balance(&contract_id));

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "gas_paid"),
            sender,
            destination_chain,
            destination_address,
            env.crypto().keccak256(&payload),
            spender,
            token,
        ),
        (Bytes::new(&env),),
    );
}

#[test]
fn fail_add_gas_zero_gas_amount() {
    let (env, _, _, client) = setup_env();

    let asset = env.register_stellar_asset_contract_v2(Address::generate(&env));

    let sender: Address = Address::generate(&env);
    let message_id = message_id(&env);
    let spender: Address = Address::generate(&env);
    let gas_amount: i128 = 0;
    let token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    assert_contract_err!(
        client.try_add_gas(&sender, &message_id, &spender, &token,),
        ContractError::InvalidAmount
    );
}

#[test]
fn fail_add_gas_not_enough_user_balance() {
    let (env, _, _, client) = setup_env();

    let asset = env.register_stellar_asset_contract_v2(Address::generate(&env));
    let sender: Address = Address::generate(&env);
    let message_id = message_id(&env);
    let spender: Address = Address::generate(&env);
    let gas_amount: i128 = 2;
    let token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    StellarAssetClient::new(&env, &asset.address()).mint(&sender, &(gas_amount - 1));

    assert!(client
        .try_add_gas(&sender, &message_id, &spender, &token,)
        .is_err());
}

#[test]
fn add_gas() {
    let (env, contract_id, _, client) = setup_env();

    let asset = env.register_stellar_asset_contract_v2(Address::generate(&env));
    let sender: Address = Address::generate(&env);
    let message_id = message_id(&env);
    let spender: Address = Address::generate(&env);
    let gas_amount: i128 = 1;
    let token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    let token_client = TokenClient::new(&env, &asset.address());
    StellarAssetClient::new(&env, &asset.address()).mint(&spender, &gas_amount);

    client.add_gas(&sender, &message_id, &spender, &token);

    assert_eq!(0, token_client.balance(&spender));
    assert_eq!(gas_amount, token_client.balance(&contract_id));

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "gas_added"),
            sender,
            message_id,
            spender,
            token,
        ),
        (),
    );
}

#[test]
fn fail_collect_fees_zero_refund_amount() {
    let (env, contract_id, gas_collector, client) = setup_env();

    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));

    let supply: i128 = 1000;
    let refund_amount = 0;

    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    StellarAssetClient::new(&env, &token.address).mint(&contract_id, &supply);

    assert_contract_err!(
        client.try_collect_fees(&gas_collector, &token),
        ContractError::InvalidAmount
    );
}

#[test]
fn fail_collect_fees_insufficient_balance() {
    let (env, contract_id, gas_collector, client) = setup_env();

    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));

    let supply: i128 = 5;
    let refund_amount = 10;

    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    StellarAssetClient::new(&env, &token.address).mint(&contract_id, &supply);

    assert_contract_err!(
        client.try_collect_fees(&gas_collector, &token),
        ContractError::InsufficientBalance
    );
}

#[test]
fn fail_collect_fees_unauthorized() {
    let (env, contract_id, _, client) = setup_env();

    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));

    let supply: i128 = 1000;
    let refund_amount = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    let user: Address = Address::generate(&env);

    StellarAssetClient::new(&env, &token.address).mint(&contract_id, &supply);

    assert_invoke_auth_err!(user, client.try_collect_fees(&user, &token));
}

#[test]
fn collect_fees() {
    let (env, contract_id, gas_collector, client) = setup_env();

    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));

    let token_client = TokenClient::new(&env, &asset.address());
    let supply: i128 = 1000;
    let refund_amount = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    StellarAssetClient::new(&env, &token.address).mint(&contract_id, &supply);

    client.collect_fees(&gas_collector, &token);

    assert_eq!(refund_amount, token_client.balance(&gas_collector));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));

    assert_last_emitted_event(
        &env,
        &contract_id,
        (Symbol::new(&env, "gas_collected"), gas_collector, token),
        (),
    );
}

#[test]
fn fail_refund_unauthorized() {
    let (env, contract_id, _, client) = setup_env();

    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));

    let supply: i128 = 1000;
    StellarAssetClient::new(&env, &asset.address()).mint(&contract_id, &supply);

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };
    let message_id = message_id(&env);
    let user: Address = Address::generate(&env);

    assert_invoke_auth_err!(user, client.try_refund(&message_id, &receiver, &token));
}

#[test]
fn fail_refund_not_enough_balance() {
    let (env, contract_id, _, client) = setup_env();

    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));

    let supply: i128 = 1;
    StellarAssetClient::new(&env, &asset.address()).mint(&contract_id, &(supply));

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 2;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    let message_id = message_id(&env);

    assert!(client.try_refund(&message_id, &receiver, &token).is_err());
}

#[test]
fn refund() {
    let (env, contract_id, _, client) = setup_env();

    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));

    let token_client = TokenClient::new(&env, &asset.address());
    let supply: i128 = 1000;
    StellarAssetClient::new(&env, &asset.address()).mint(&contract_id, &supply);

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 1;
    let token = Token {
        address: asset.address(),
        amount: refund_amount,
    };

    let message_id = message_id(&env);

    client.refund(&message_id, &receiver, &token);

    assert_eq!(refund_amount, token_client.balance(&receiver));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "gas_refunded"),
            message_id,
            receiver,
            token,
        ),
        (),
    );
}
