use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};
use stellar_axelar_std::assert_auth;
use stellar_axelar_std::interfaces::PausableClient;
use stellar_axelar_std_derive::{Ownable, Pausable};

use crate as stellar_axelar_std;

#[contract]
#[derive(Ownable, Pausable)]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn __constructor(env: &Env, owner: Address) {
        stellar_axelar_std::interfaces::set_owner(env, &owner);
    }
}

#[test]
fn contract_pause_unpause_succeeds() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let contract_id = env.register(Contract, (owner.clone(),));
    let client = PausableClient::new(&env, &contract_id);

    assert!(!client.paused());

    assert_auth!(owner, client.pause());
    assert!(client.paused());

    assert_auth!(owner, client.unpause());
    assert!(!client.paused());
}
