use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};
use stellar_axelar_std::interfaces::OwnableClient;
use stellar_axelar_std::{assert_auth, assert_auth_err, only_owner};
use stellar_axelar_std_derive::Ownable;

use crate as stellar_axelar_std;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
enum ContractError {
    MigrationNotAllowed = 1,
}

#[contract]
#[derive(Ownable)]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn __constructor(env: &Env, owner: Address) {
        stellar_axelar_std::interfaces::set_owner(env, &owner);
    }

    #[only_owner]
    pub fn owner_function(env: &Env) {}
}

#[test]
fn owner_function_succeeds_with_correct_owner() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let contract_id = env.register(Contract, (owner.clone(),));
    let client = ContractClient::new(&env, &contract_id);

    assert_auth!(owner, client.owner_function());
}

#[test]
fn owner_function_fails_with_incorrect_owner() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let contract_id = env.register(Contract, (owner,));
    let client = ContractClient::new(&env, &contract_id);

    assert_auth_err!(non_owner, client.owner_function());
}

#[test]
fn contract_ownership_transfer_succeeds() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let contract_id = env.register(Contract, (owner.clone(),));
    let client = OwnableClient::new(&env, &contract_id);
    assert_eq!(owner, client.owner());

    let new_owner = Address::generate(&env);
    assert_auth!(owner, client.transfer_ownership(&new_owner));
    assert_eq!(new_owner, client.owner());
}
