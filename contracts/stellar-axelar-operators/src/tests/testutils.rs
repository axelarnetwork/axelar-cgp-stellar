use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

use super::test::TestTarget;
use crate::{AxelarOperators, AxelarOperatorsClient};

pub struct TestConfig<'a> {
    pub env: Env,
    pub owner: Address,
    pub client: AxelarOperatorsClient<'a>,
    pub target_id: Address,
}

pub fn setup_env<'a>() -> TestConfig<'a> {
    let env = Env::default();

    let owner = Address::generate(&env);
    let contract_id = env.register(AxelarOperators, (&owner,));
    let client = AxelarOperatorsClient::new(&env, &contract_id);

    let target_id = env.register(TestTarget, ());

    TestConfig {
        env,
        owner,
        client,
        target_id,
    }
}
