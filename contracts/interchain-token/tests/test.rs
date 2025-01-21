#![cfg(test)]
extern crate std;

use soroban_sdk::testutils::{Address as _, BytesN as _, Ledger};
use soroban_sdk::{Address, BytesN, Env, IntoVal as _, Symbol};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_std::{assert_auth, assert_auth_err, assert_last_emitted_event};
use stellar_interchain_token::{InterchainToken, InterchainTokenClient};

fn setup_token_metadata(env: &Env, name: &str, symbol: &str, decimal: u32) -> TokenMetadata {
    TokenMetadata {
        decimal,
        name: name.into_val(env),
        symbol: symbol.into_val(env),
    }
}

fn setup_token<'a>(env: &Env) -> (InterchainTokenClient<'a>, Address, Address) {
    let owner = Address::generate(env);
    let minter = Address::generate(env);
    let token_id: BytesN<32> = BytesN::<32>::random(env);
    let token_metadata = setup_token_metadata(env, "name", "symbol", 6);

    let contract_id = env.register(
        InterchainToken,
        (owner.clone(), minter.clone(), &token_id, token_metadata),
    );

    let token = InterchainTokenClient::new(env, &contract_id);
    (token, owner, minter)
}

#[test]
fn register_interchain_token() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_metadata = setup_token_metadata(&env, "name", "symbol", 6);

    let contract_id = env.register(
        InterchainToken,
        (
            owner.clone(),
            minter.clone(),
            &token_id,
            token_metadata.clone(),
        ),
    );

    let token = InterchainTokenClient::new(&env, &contract_id);

    assert_eq!(token.token_id(), token_id);
    assert_eq!(token.name(), token_metadata.name);
    assert_eq!(token.symbol(), token_metadata.symbol);
    assert_eq!(token.decimals(), token_metadata.decimal);
    assert_eq!(token.owner(), owner);
    assert!(token.is_minter(&owner));
    assert!(token.is_minter(&minter));
}

#[test]
fn register_interchain_token_without_minter() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_metadata = setup_token_metadata(&env, "name", "symbol", 6);
    let minter: Option<Address> = None;

    let contract_id = env.register(
        InterchainToken,
        (owner.clone(), minter, &token_id, token_metadata),
    );

    let token = InterchainTokenClient::new(&env, &contract_id);

    assert_eq!(token.owner(), owner);
    assert!(token.is_minter(&owner));
}

#[test]
fn transfer_ownership_from_non_owner_fails() {
    let env = Env::default();

    let new_owner = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, _owner, _minter) = setup_token(&env);

    assert_auth_err!(user, token.transfer_ownership(&new_owner));
}

#[test]
fn transfer_ownership() {
    let env = Env::default();
    let new_owner = Address::generate(&env);

    let (token, owner, _minter) = setup_token(&env);

    assert_eq!(token.owner(), owner);

    assert_auth!(owner, token.transfer_ownership(&new_owner));

    assert_eq!(token.owner(), new_owner);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")] // NegativeAmount
fn transfer_fails_with_negative_amount() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount = -1;

    let (token, _owner, _minter) = setup_token(&env);

    token.mock_all_auths().transfer(&user1, &user2, &amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")] // InsufficientBalance
fn transfer_fails_with_insufficient_balance() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount = 1000;

    let (token, _owner, _minter) = setup_token(&env);

    token.mock_all_auths().transfer(&user1, &user2, &amount);
}

#[test]
fn transfer() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount = 1000;

    let (token, _owner, minter) = setup_token(&env);

    assert_auth!(minter, token.mint_from(&minter, &user1, &amount));
    assert_eq!(token.balance(&user1), amount);

    assert_auth!(user1, token.transfer(&user1, &user2, &600_i128));
    assert_eq!(token.balance(&user1), 400_i128);
    assert_eq!(token.balance(&user2), 600_i128);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")] // NegativeAmount
fn transfer_from_fails_with_negative_amount() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let amount = -1;

    let (token, _owner, minter) = setup_token(&env);

    assert_auth!(minter, token.mint_from(&minter, &user1, &1000_i128));
    assert_eq!(token.balance(&user1), 1000_i128);

    let expiration_ledger = 200;

    assert_auth!(
        user1,
        token.approve(&user1, &user2, &500_i128, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user1, &user2), 500_i128);

    token
        .mock_all_auths()
        .transfer_from(&user2, &user1, &user3, &amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")] // InsufficientAllowance
fn transfer_from_fails_without_approval() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);

    assert_auth!(minter, token.mint_from(&minter, &user1, &1000_i128));
    assert_eq!(token.balance(&user1), 1000_i128);

    token
        .mock_all_auths()
        .transfer_from(&user2, &user1, &user3, &400_i128);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")] // InsufficientAllowance
fn transfer_from_fails_with_insufficient_allowance() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);

    assert_auth!(minter, token.mint_from(&minter, &user1, &1000_i128));
    assert_eq!(token.balance(&user1), 1000_i128);

    let expiration_ledger = 200;

    assert_auth!(
        user1,
        token.approve(&user1, &user2, &100_i128, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user1, &user2), 100_i128);

    token
        .mock_all_auths()
        .transfer_from(&user2, &user1, &user3, &400_i128);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")] // InsufficientAllowance
fn transfer_from_fails_with_expired_allowance() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);

    token
        .mock_all_auths()
        .mint_from(&minter, &user1, &1000_i128);
    let allowance: i128 = 100;

    let current_ledger = env.ledger().sequence();
    let expiration_ledger = current_ledger + 100;

    token
        .mock_all_auths()
        .approve(&user1, &user2, &allowance, &expiration_ledger);

    env.ledger().set_sequence_number(expiration_ledger + 1);

    token
        .mock_all_auths()
        .transfer_from(&user2, &user1, &user3, &allowance);
}

#[test]
fn transfer_from() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);

    assert_auth!(minter, token.mint_from(&minter, &user1, &1000_i128));
    assert_eq!(token.balance(&user1), 1000_i128);

    let expiration_ledger = 200;

    assert_auth!(
        user1,
        token.approve(&user1, &user2, &500_i128, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user1, &user2), 500_i128);

    assert_auth!(
        user2,
        token.transfer_from(&user2, &user1, &user3, &400_i128)
    );
    assert_eq!(token.balance(&user1), 600_i128);
    assert_eq!(token.balance(&user2), 0_i128);
    assert_eq!(token.balance(&user3), 400_i128);
}

#[test]
fn mint_from_invalid_minter_fails() {
    let env = Env::default();

    let amount = 1000;

    let user = Address::generate(&env);

    let (token, owner, minter) = setup_token(&env);

    assert_auth_err!(owner, token.mint_from(&minter, &user, &amount));
    assert_auth_err!(user, token.mint_from(&minter, &user, &amount));
    assert_auth_err!(user, token.mint(&user, &amount));
}

#[test]
fn mint_from_minter_succeeds() {
    let env = Env::default();

    let amount = 1000;
    let user = Address::generate(&env);

    let (token, owner, minter) = setup_token(&env);

    assert_auth!(minter, token.mint_from(&minter, &user, &amount));
    assert_eq!(token.balance(&user), amount);

    assert_auth!(owner, token.mint(&user, &amount));
    assert_eq!(token.balance(&user), amount * 2);
}

#[test]
fn add_minter_fails_without_owner_auth() {
    let env = Env::default();

    let minter2 = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, _owner, _minter1) = setup_token(&env);

    assert_auth_err!(user, token.add_minter(&minter2));
}

#[test]
fn add_minter_succeeds() {
    let env = Env::default();

    let amount = 1000;
    let minter2 = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, owner, _minter1) = setup_token(&env);

    assert_auth!(owner, token.add_minter(&minter2));

    assert_last_emitted_event(
        &env,
        &token.address,
        (Symbol::new(&env, "minter_added"), minter2.clone()),
        (),
    );

    assert_auth!(minter2, token.mint_from(&minter2, &user, &amount));
    assert_eq!(token.balance(&user), amount);
}

#[test]
fn remove_minter_fails_without_owner_auth() {
    let env = Env::default();

    let minter1 = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, _owner, _minter) = setup_token(&env);

    assert_auth_err!(user, token.remove_minter(&minter1));
}

#[test]
fn remove_minter() {
    let env = Env::default();

    let amount = 1000;
    let minter1 = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, owner, _minter) = setup_token(&env);

    assert_auth!(owner, token.remove_minter(&minter1));

    assert_last_emitted_event(
        &env,
        &token.address,
        (Symbol::new(&env, "minter_removed"), minter1.clone()),
        (),
    );

    assert_auth_err!(minter1, token.mint_from(&minter1, &user, &amount));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")] // NegativeAmount
fn burn_fails_with_negative_amount() {
    let env = Env::default();

    let user = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_auth!(minter, token.mint_from(&minter, &user, &amount));
    assert_eq!(token.balance(&user), amount);

    let burn_amount = -1;

    token.mock_all_auths().burn(&user, &burn_amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")] // InsufficientBalance
fn burn_fails_with_insufficient_balance() {
    let env = Env::default();

    let user = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_auth!(minter, token.mint_from(&minter, &user, &amount));
    assert_eq!(token.balance(&user), amount);

    let burn_amount = 2000;

    token.mock_all_auths().burn(&user, &burn_amount);
}

#[test]
fn burn_succeeds() {
    let env = Env::default();

    let user = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_auth!(minter, token.mint_from(&minter, &user, &amount));
    assert_eq!(token.balance(&user), amount);

    assert_auth!(user, token.burn(&user, &amount));
    assert_eq!(token.balance(&user), 0);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")] // NegativeAmount
fn burn_from_fails_with_negative_amount() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token, _owner, _minter) = setup_token(&env);

    let burn_amount = -1;

    token
        .mock_all_auths()
        .burn_from(&user2, &user1, &burn_amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")] // InsufficientAllowance
fn burn_from_fails_without_approval() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_auth!(minter, token.mint_from(&minter, &user1, &amount));
    assert_eq!(token.balance(&user1), amount);

    let burn_amount = 500;

    token
        .mock_all_auths()
        .burn_from(&user2, &user1, &burn_amount);
}

#[test]
fn burn_from_succeeds() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_auth!(minter, token.mint_from(&minter, &user1, &amount));
    assert_eq!(token.balance(&user1), amount);

    let expiration_ledger = 200;
    let burn_amount = 100;

    assert_auth!(
        user1,
        token.approve(&user1, &user2, &burn_amount, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user1, &user2), burn_amount);

    assert_auth!(user2, token.burn_from(&user2, &user1, &burn_amount));
    assert_eq!(token.allowance(&user1, &user2), 0);
    assert_eq!(token.balance(&user1), (amount - burn_amount));
    assert_eq!(token.balance(&user2), 0);
}
