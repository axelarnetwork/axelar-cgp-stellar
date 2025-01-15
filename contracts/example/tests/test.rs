#![cfg(test)]
extern crate std;

use example::{Example, ExampleClient};
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _};
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, IntoVal, String, Symbol};
use stellar_axelar_gas_service::{AxelarGasService, AxelarGasServiceClient};
use stellar_axelar_gateway::{
    testutils::{self, generate_proof, get_approve_hash, TestSignerSet},
    types::Message,
    AxelarGatewayClient,
};
use stellar_axelar_std::{assert_last_emitted_event, auth_invocation, types::Token};

fn setup_gateway<'a>(env: &Env) -> (TestSignerSet, AxelarGatewayClient<'a>) {
    let (signers, client) = testutils::setup_gateway(env, 0, 5);
    (signers, client)
}

fn setup_gas_service<'a>(env: &Env) -> (AxelarGasServiceClient<'a>, Address, Address) {
    let owner: Address = Address::generate(env);
    let gas_collector: Address = Address::generate(env);
    let gas_service_id = env.register(AxelarGasService, (&owner, &gas_collector));
    let gas_service_client = AxelarGasServiceClient::new(env, &gas_service_id);

    (gas_service_client, gas_collector, gas_service_id)
}

fn setup_app<'a>(env: &Env, gateway: &Address, gas_service: &Address) -> ExampleClient<'a> {
    let id = env.register(Example, (gateway, gas_service));
    let client = ExampleClient::new(env, &id);

    client
}

#[test]
fn gmp_example() {
    let env = Env::default();
    env.mock_all_auths();

    let user: Address = Address::generate(&env);

    // Setup source Axelar gateway
    let source_chain = String::from_str(&env, "source");
    let (_, source_gateway_client) = setup_gateway(&env);
    let source_gateway_id = source_gateway_client.address;
    let (source_gas_service_client, _source_gas_collector, source_gas_service_id) =
        setup_gas_service(&env);
    let source_app = setup_app(&env, &source_gateway_id, &source_gas_service_id);

    // Setup destination Axelar gateway
    let destination_chain = String::from_str(&env, "destination");
    let (signers, destination_gateway_client) = setup_gateway(&env);

    let (_destination_gas_service_client, _destination_gas_collector, destination_gas_service_id) =
        setup_gas_service(&env);
    let destination_app = setup_app(
        &env,
        &destination_gateway_client.address,
        &destination_gas_service_id,
    );

    // Set cross-chain message params
    let source_address = source_app.address.to_string();
    let destination_address = destination_app.address.to_string();
    let payload: Bytes = BytesN::<20>::random(&env).into();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    // Initiate cross-chain contract call, sending message from source to destination
    let asset = &env.register_stellar_asset_contract_v2(user.clone());
    let asset_client = StellarAssetClient::new(&env, &asset.address());
    let gas_amount: i128 = 100;
    let gas_token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    asset_client.mint(&user, &gas_amount);

    source_app.send(
        &user,
        &destination_chain,
        &destination_address,
        &payload,
        &gas_token,
    );

    let transfer_auth = auth_invocation!(
        &env,
        user,
        asset_client.transfer(&user, source_gas_service_id, gas_token.amount)
    );

    let pay_gas_auth = auth_invocation!(
        &env,
        user,
        source_gas_service_client.pay_gas(
            source_app.address.clone(),
            destination_chain.clone(),
            destination_address.clone(),
            payload.clone(),
            &user,
            gas_token.clone(),
            &Bytes::new(&env)
        ),
        transfer_auth
    );

    let send_auth = auth_invocation!(
        &env,
        user,
        source_app.send(
            &user,
            destination_chain.clone(),
            destination_address.clone(),
            payload.clone(),
            gas_token
        ),
        pay_gas_auth
    );

    assert_eq!(env.auths(), send_auth);

    // Axelar hub confirms the contract call, i.e Axelar verifiers verify/vote on the emitted event
    let message_id = String::from_str(&env, "test");

    // Confirming message from source Axelar gateway
    assert_last_emitted_event(
        &env,
        &source_gateway_id,
        (
            Symbol::new(&env, "contract_called"),
            source_app.address,
            destination_chain,
            destination_address,
            payload_hash.clone(),
        ),
        (payload.to_val(),),
    );

    // Axelar hub signs the message approval, Signing message approval for destination
    let messages = vec![
        &env,
        Message {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
            source_address: source_address.clone(),
            contract_address: destination_app.address.clone(),
            payload_hash,
        },
    ];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    // Submitting signed message approval to destination Axelar gateway
    destination_gateway_client.approve_messages(&messages, &proof);

    // Executing message on destination app
    destination_app.execute(&source_chain, &message_id, &source_address, &payload);

    assert_last_emitted_event(
        &env,
        &destination_app.address,
        (
            Symbol::new(&env, "executed"),
            source_chain,
            message_id,
            source_address,
        ),
        (payload,),
    );
}
