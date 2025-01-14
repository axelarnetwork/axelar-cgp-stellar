#![cfg(test)]
extern crate std;

use stellar_axelar_soroban_std::{
    assert_invoke_auth_err, assert_invoke_auth_ok, assert_last_emitted_event,
};

use stellar_interchain_token::{InterchainToken, InterchainTokenClient};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, IntoVal as _, Symbol,
};
use soroban_token_sdk::metadata::TokenMetadata;

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
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn register_token_with_invalid_decimals_fails() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_metadata = setup_token_metadata(&env, "name", "symbol", u32::from(u8::MAX) + 1);

    env.register(InterchainToken, (owner, minter, &token_id, token_metadata));
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn register_token_with_invalid_name_fails() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_metadata = setup_token_metadata(&env, "", "symbol", 1);

    env.register(InterchainToken, (owner, minter, &token_id, token_metadata));
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn register_token_with_invalid_symbol_fails() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_metadata = setup_token_metadata(&env, "name", "", 1);

    env.register(InterchainToken, (owner, minter, &token_id, token_metadata));
}

#[test]
fn register_interchain_token() {
    let env = Env::default();

    let (token, owner, minter) = setup_token(&env);

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
fn transfer_ownership_from_non_owner() {
    let env = Env::default();

    let new_owner = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, _owner, _minter) = setup_token(&env);

    assert_invoke_auth_err!(user, token.try_transfer_ownership(&new_owner));
}

#[test]
fn transfer_ownership() {
    let env = Env::default();
    let new_owner = Address::generate(&env);

    let (token, owner, _minter) = setup_token(&env);

    assert_eq!(token.owner(), owner);

    assert_invoke_auth_ok!(owner, token.try_transfer_ownership(&new_owner));

    assert_eq!(token.owner(), new_owner);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")] // NegativeAmount
fn fail_transfer_with_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount = -1;

    let (token, _owner, _minter) = setup_token(&env);

    token.transfer(&user1, &user2, &amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")] // InsufficientBalance
fn fail_transfer_with_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount = 1000;

    let (token, _owner, _minter) = setup_token(&env);

    token.transfer(&user1, &user2, &amount);
}

#[test]
fn transfer() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount = 1000;

    let (token, _owner, minter) = setup_token(&env);

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user1, &amount));
    assert_eq!(token.balance(&user1), amount);

    assert_invoke_auth_ok!(user1, token.try_transfer(&user1, &user2, &600_i128));
    assert_eq!(token.balance(&user1), 400_i128);
    assert_eq!(token.balance(&user2), 600_i128);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")] // NegativeAmount
fn fail_transfer_from_with_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let amount = -1;

    let (token, _owner, minter) = setup_token(&env);

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user1, &1000_i128));
    assert_eq!(token.balance(&user1), 1000_i128);

    let expiration_ledger = 200;

    assert_invoke_auth_ok!(
        user1,
        token.try_approve(&user1, &user2, &500_i128, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user1, &user2), 500_i128);

    token.transfer_from(&user2, &user1, &user3, &amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")] // InsufficientAllowance
fn fail_transfer_from_without_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user1, &1000_i128));
    assert_eq!(token.balance(&user1), 1000_i128);

    token.transfer_from(&user2, &user1, &user3, &400_i128);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")] // InsufficientAllowance
fn fail_transfer_from_with_insufficient_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user1, &1000_i128));
    assert_eq!(token.balance(&user1), 1000_i128);

    let expiration_ledger = 200;

    assert_invoke_auth_ok!(
        user1,
        token.try_approve(&user1, &user2, &100_i128, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user1, &user2), 100_i128);

    token.transfer_from(&user2, &user1, &user3, &400_i128);
}

#[test]
fn transfer_from() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user1, &1000_i128));
    assert_eq!(token.balance(&user1), 1000_i128);

    let expiration_ledger = 200;

    assert_invoke_auth_ok!(
        user1,
        token.try_approve(&user1, &user2, &500_i128, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user1, &user2), 500_i128);

    assert_invoke_auth_ok!(
        user2,
        token.try_transfer_from(&user2, &user1, &user3, &400_i128)
    );
    assert_eq!(token.balance(&user1), 600_i128);
    assert_eq!(token.balance(&user2), 0_i128);
    assert_eq!(token.balance(&user3), 400_i128);
}

#[test]
fn fail_mint_from_invalid_minter() {
    let env = Env::default();

    let amount = 1000;

    let user = Address::generate(&env);

    let (token, owner, minter) = setup_token(&env);

    assert_invoke_auth_err!(owner, token.try_mint_from(&minter, &user, &amount));
    assert_invoke_auth_err!(user, token.try_mint_from(&minter, &user, &amount));
    assert_invoke_auth_err!(user, token.try_mint(&user, &amount));
}

#[test]
fn mint_from_minter_succeeds() {
    let env = Env::default();

    let amount = 1000;
    let user = Address::generate(&env);

    let (token, owner, minter) = setup_token(&env);

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user, &amount));
    assert_eq!(token.balance(&user), amount);

    assert_invoke_auth_ok!(owner, token.try_mint(&user, &amount));
    assert_eq!(token.balance(&user), amount * 2);
}

#[test]
fn fail_add_minter_from_non_owner() {
    let env = Env::default();

    let minter2 = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, _owner, _minter1) = setup_token(&env);

    assert_invoke_auth_err!(user, token.try_add_minter(&minter2));
}

#[test]
fn add_minter_succeeds() {
    let env = Env::default();

    let amount = 1000;
    let minter2 = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, owner, _minter1) = setup_token(&env);

    assert_invoke_auth_ok!(owner, token.try_add_minter(&minter2));

    assert_last_emitted_event(
        &env,
        &token.address,
        (Symbol::new(&env, "minter_added"), minter2.clone()),
        (),
    );

    assert_invoke_auth_ok!(minter2, token.try_mint_from(&minter2, &user, &amount));
    assert_eq!(token.balance(&user), amount);
}

#[test]
fn fail_remove_minter_from_non_owner() {
    let env = Env::default();

    let minter1 = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, _owner, _minter) = setup_token(&env);

    assert_invoke_auth_err!(user, token.try_remove_minter(&minter1));
}

#[test]
fn remove_minter() {
    let env = Env::default();

    let amount = 1000;
    let minter1 = Address::generate(&env);
    let user = Address::generate(&env);

    let (token, owner, _minter) = setup_token(&env);

    assert_invoke_auth_ok!(owner, token.try_remove_minter(&minter1));

    assert_last_emitted_event(
        &env,
        &token.address,
        (Symbol::new(&env, "minter_removed"), minter1.clone()),
        (),
    );

    assert_invoke_auth_err!(minter1, token.try_mint_from(&minter1, &user, &amount));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")] // NegativeAmount
fn fail_burn_with_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user, &amount));
    assert_eq!(token.balance(&user), amount);

    let burn_amount = -1;

    token.burn(&user, &burn_amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")] // InsufficientBalance
fn fail_burn_with_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user, &amount));
    assert_eq!(token.balance(&user), amount);

    let burn_amount = 2000;

    token.burn(&user, &burn_amount);
}

#[test]
fn burn_succeeds() {
    let env = Env::default();

    let user = Address::generate(&env);

    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user, &amount));
    assert_eq!(token.balance(&user), amount);

    assert_invoke_auth_ok!(user, token.try_burn(&user, &amount));
    assert_eq!(token.balance(&user), 0);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")] // NegativeAmount
fn fail_burn_from_with_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token, _owner, _minter) = setup_token(&env);

    let burn_amount = -1;

    token.burn_from(&user2, &user1, &burn_amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")] // InsufficientAllowance
fn fail_burn_from_without_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user1, &amount));
    assert_eq!(token.balance(&user1), amount);

    let burn_amount = 500;

    token.burn_from(&user2, &user1, &burn_amount);
}

#[test]
fn burn_from_succeeds() {
    let env = Env::default();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token, _owner, minter) = setup_token(&env);
    let amount = 1000;

    assert_invoke_auth_ok!(minter, token.try_mint_from(&minter, &user1, &amount));
    assert_eq!(token.balance(&user1), amount);

    let expiration_ledger = 200;
    let burn_amount = 100;

    assert_invoke_auth_ok!(
        user1,
        token.try_approve(&user1, &user2, &burn_amount, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user1, &user2), burn_amount);

    assert_invoke_auth_ok!(user2, token.try_burn_from(&user2, &user1, &burn_amount));
    assert_eq!(token.allowance(&user1, &user2), 0);
    assert_eq!(token.balance(&user1), (amount - burn_amount));
    assert_eq!(token.balance(&user2), 0);
}
