use soroban_sdk::{Address, Env, String};

use crate::migrate::legacy_storage;
use crate::storage::MessageApprovalValue;
use crate::testutils::{setup_gateway, TestSignerSet};
use crate::AxelarGatewayClient;

pub struct TestConfig<'a> {
    pub env: Env,
    pub signers: TestSignerSet,
    pub client: AxelarGatewayClient<'a>,
}

pub fn setup_env<'a>(previous_signers_retention: u64, num_signers: u64) -> TestConfig<'a> {
    let env = Env::default();
    let (signers, client) = setup_gateway(&env, previous_signers_retention, num_signers);

    TestConfig {
        env,
        signers,
        client,
    }
}

pub fn setup_legacy_message_approval(
    env: &Env,
    source_chain: String,
    message_id: String,
    value: MessageApprovalValue,
) {
    let key = legacy_storage::MessageApprovalKey {
        source_chain,
        message_id,
    };

    legacy_storage::set_message_approval(env, key, &value);
}

pub fn get_message_approval(
    env: &Env,
    contract_id: &Address,
    source_chain: &String,
    message_id: &String,
) -> MessageApprovalValue {
    env.as_contract(contract_id, || {
        crate::storage::message_approval(env, source_chain.clone(), message_id.clone())
    })
}
