use axelar_gateway::testutils::{setup_gateway, TestSignerSet};
use axelar_gateway::AxelarGatewayClient;
use soroban_sdk::Env;

pub fn setup_env<'a>(
    previous_signers_retention: u32,
    num_signers: u32,
) -> (Env, TestSignerSet, AxelarGatewayClient<'a>) {
    let env = Env::default();
    let (signers, client) = setup_gateway(&env, previous_signers_retention, num_signers);

    (env, signers, client)
}
