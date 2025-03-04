use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};
use stellar_axelar_std::interfaces::OperatableClient;
use stellar_axelar_std::{assert_auth, assert_auth_err, only_operator};
use stellar_axelar_std_derive::Operatable;

use crate as stellar_axelar_std;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
enum ContractError {
    MigrationNotAllowed = 1,
}

#[contract]
#[derive(Operatable)]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn __constructor(env: &Env, operator: Address) {
        stellar_axelar_std::interfaces::set_operator(env, &operator);
    }

    #[only_operator]
    pub fn operator_function(env: &Env) {}
}

#[test]
fn operator_function_succeeds_with_correct_operator() {
    let env = Env::default();
    let operator = Address::generate(&env);
    let contract_id = env.register(Contract, (operator.clone(),));
    let client = ContractClient::new(&env, &contract_id);

    assert_auth!(operator, client.operator_function());
}

#[test]
fn operator_function_fails_with_incorrect_operator() {
    let env = Env::default();
    let operator = Address::generate(&env);
    let non_operator = Address::generate(&env);
    let contract_id = env.register(Contract, (operator,));
    let client = ContractClient::new(&env, &contract_id);

    assert_auth_err!(non_operator, client.operator_function());
}

#[test]
fn contract_operatorship_transfer_succeeds() {
    let env = Env::default();
    let operator = Address::generate(&env);
    let contract_id = env.register(Contract, (operator.clone(),));
    let client = OperatableClient::new(&env, &contract_id);
    assert_eq!(operator, client.operator());

    let new_operator = Address::generate(&env);
    assert_auth!(operator, client.transfer_operatorship(&new_operator));
    assert_eq!(new_operator, client.operator());
}
