#![cfg(test)]

extern crate std;

use core::f32::consts::E;

use crate::{gateway::*, GatewayClient};
use soroban_sdk::{bytes, bytesn, log, testutils::{Address as _, Events}, vec, xdr::ToXdr, Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Vec};

use rand::rngs::OsRng;
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};

//#[test]
fn approve_contract_call() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // Data for Contract Approve
    let params_approve = ContractPayload {
        src_chain: String::from_str(&env, "0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d"),
        src_addr: String::from_str(&env, "0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d"),
        contract: String::from_str(&env, "0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d"),
        payload_ha: bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_tx_ha: bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_evnt: 1, // source event index // do u256 instead?
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x617070726f7665436f6e747261637443616c6c]], // approveContractCall converted into Bytes, and then keccak256 hashed.
        params: vec![&env, params_approve.clone().to_xdr(&env)],
    };

    const THRESHOLD: u128 = 10;
    let weights: Vec<u128> = vec![&env, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    const NUM_OPS: u32 = 10;

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let signers: Vec<[u8; 32]> = keypairs.clone();

    let proof: Validate = generate_test_proof(env.clone(), data.clone(), keypairs.clone(), weights, THRESHOLD, signers.clone());

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env),
    };

    // Initalize
    let initialize_ops: Operatorship = Operatorship {
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &initialize_ops.clone().to_xdr(&env));

    // test Approve Contract Call
    let test = input.to_xdr(&env);
    client.execute(&test);


    let event0: Operatorship = initialize_ops;
    let event1: ContractCallApprovedEvent = ContractCallApprovedEvent { src_chain: params_approve.src_chain, src_addr: params_approve.src_addr, src_tx: params_approve.src_tx_ha, src_event: params_approve.src_evnt };
    let event2: ExecutedEvent = ExecutedEvent { command_id: data.commandids.get(0).unwrap() };
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                ().into_val(&env),
                event0.into_val(&env)
            ),
            (
                contract_id.clone(),
                (
                    bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
                    bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
                    bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
                ).into_val(&env),
                event1.into_val(&env)
            ),
            (
                contract_id.clone(),
                ().into_val(&env),
                event2.into_val(&env)
            ),
        ]
    );
}

#[test]
fn call_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // Test Call Contract
    let user: Address = Address::generate(&env);
    let ethereum_id: Bytes = bytes!(&env, 0x0);
    let junkyard: Bytes = bytes!(&env, 0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59);
    let payload: Bytes = bytes!(&env, 0x000000000000000000000000da2982fa68c3787af86475824eeb07702c4c449f00000000000000000000000000000000000000000000000000000000000003be0000000000000000000000004efe356bedecc817cb89b4e9b796db8bc188dc59);
    client.call_contract(
        &user,
        &ethereum_id,
        &junkyard,
        &payload,
    );

    let event: ContractCall = ContractCall {
        prefix: Symbol::new(&env, "ContractCall"),
        dest_chain: ethereum_id,
        dest_addr: junkyard,
        payload: payload.clone(),
    };
    let events = env.events().all();

    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id.clone(),
                (
                    user,
                    env.crypto().keccak256(&payload)
                ).into_val(&env),
                event.into_val(&env)
            ),
        ]
    );
}

// 'validate the proof from the current operators' is tested indirectly.
//#[test]
fn transfer_operatorship() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    let new_operators: Operatorship = Operatorship {
        new_ops: vec![&env,
                      bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001),
                      bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002),
        ],
        new_wghts: vec![&env, 1, 1],
        new_thres: 2,
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // transferOperatorship converted into Bytes,
        params: vec![&env, new_operators.clone().to_xdr(&env)],
    };

    const THRESHOLD: u128 = 4;
    const NUM_OPS: u32 = 3;

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let mut signers: Vec<[u8; 32]> = keypairs.clone();
    signers.remove(1);

    let proof: Validate = generate_test_proof(env.clone(), data.clone(), keypairs.clone(), vec![&env, 1, 1, 3], THRESHOLD, signers.clone());

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env),
    };

    // Initalize with 3 random operators
    let params_operator: Operatorship = Operatorship {
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));


    // Transfer operatorship to 2 new operators in the variable new_operators
    let test = input.to_xdr(&env);
    client.execute(&test);


    let initialize_ops: Operatorship = params_operator;
    let new_ops: Operatorship = new_operators;
    let success: ExecutedEvent = ExecutedEvent { command_id: data.commandids.get(0).unwrap() };
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                ().into_val(&env),
                initialize_ops.into_val(&env)
            ),
            (
                contract_id.clone(),
                ().into_val(&env),
                new_ops.into_val(&env)
            ),
            (
                contract_id.clone(),
                ().into_val(&env),
                success.into_val(&env)
            ),
        ]
    );
}

// 'validate the proof for a single signer & operator.
#[test]
fn single_operator_signer() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 1;
    const THRESHOLD: u128 = 1;
    let weights: Vec<u128> = vec![&env, 1];

    let new_keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let signers: Vec<[u8; 32]> = keypairs.clone();

    let new_operatorship: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), new_keypairs.clone()),
        new_wghts: weights.clone(),
        new_thres: THRESHOLD,
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // transferOperatorship converted into Bytes
        params: vec![&env, new_operatorship.clone().to_xdr(&env)],
    };

    let proof: Validate = generate_test_proof(env.clone(), data.clone(), keypairs.clone(), weights.clone(), THRESHOLD, signers.clone());

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env),
    };

    // Initalize with 1 random operator
    let initialize_operators: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), keypairs.clone()),
        new_wghts: weights.clone(),
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &initialize_operators.clone().to_xdr(&env));
    client.execute(&input.to_xdr(&env));


    let success: ExecutedEvent = ExecutedEvent { command_id: data.commandids.get(0).unwrap() };
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                ().into_val(&env),
                initialize_operators.into_val(&env)
            ),
            (
                contract_id.clone(),
                ().into_val(&env),
                new_operatorship.into_val(&env)
            ),
            (
                contract_id.clone(),
                ().into_val(&env),
                success.into_val(&env)
            ),
        ]
    );
}

#[test]
#[should_panic]
fn no_operators() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x617070726f7665436f6e747261637443616c6c]],
        params: vec![&env, bytes![&env, 0x0]],
    };

    const THRESHOLD: u128 = 10;
    const NUM_OPS: u32 = 0;

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let signers: Vec<[u8; 32]> = keypairs.clone();

    let proof: Validate = generate_test_proof(env.clone(), data.clone(), keypairs.clone(), vec![&env], THRESHOLD, signers.clone());

    // Test Initalize
    let params_operator: Operatorship = Operatorship {
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));
}

// 'should not allow transferring operatorship to unsorted operators'
#[test]
#[should_panic]
fn operators_not_sorted() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // Test Initalize
    let params_operator: Operatorship = Operatorship {
        new_ops: vec![&env,
                      bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002),
                      bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001),
        ],
        new_wghts: vec![&env, 1, 1],
        new_thres: 1,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));
}

// 'should not allow transferring operatorship with invalid number of weights'
#[test]
#[should_panic]
fn invalid_weights() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 2;
    const THRESHOLD: u128 = 1;
    let weights: Vec<u128> = vec![&env, 1];

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let signers: Vec<[u8; 32]> = keypairs.clone();

    // Test Initalize
    let params_operator: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), keypairs.clone()),
        new_wghts: weights,
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));
}

// 'should not allow transferring operatorship with invalid threshold'
#[test]
#[should_panic]
fn invalid_threshold() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 2;
    const THRESHOLD: u128 = 3;
    let weights: Vec<u128> = vec![&env, 1, 1];

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);

    // Test Initalize
    let params_operator: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), keypairs.clone()),
        new_wghts: weights,
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));
}

// 'should not allow transferring operatorship to duplicated operators'
#[test]
#[should_panic]
fn duplicate_operators() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 2;
    const THRESHOLD: u128 = 2;
    let weights: Vec<u128> = vec![&env, 2, 1];

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let mut signers: Vec<[u8; 32]> = keypairs.clone();
    signers.remove(1);

    let new_operatorship: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), keypairs.clone()),
        new_wghts: weights.clone(),
        new_thres: THRESHOLD,
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // transferOperatorship converted into Bytes
        params: vec![&env, new_operatorship.clone().to_xdr(&env)],
    };

    let proof: Validate = generate_test_proof(env.clone(), data.clone(), keypairs.clone(), weights, THRESHOLD, signers.clone());


    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env),
    };

    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &new_operatorship.clone().to_xdr(&env));

    client.execute(&input.to_xdr(&env));
}

// 'should not allow transferring operatorship with invalid threshold'
#[test]
#[should_panic]
fn invalid_threshold_2() {
    // This case differs, as while the transfer_ops() called in initialize() passes, the transfer_ops() in execute()
    // fails as the new operators do not pass the threshold.
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 2;
    const THRESHOLD: u128 = 3;
    let weights: Vec<u128> = vec![&env, 1, 1];

    let keypairs_1: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let keypairs_2: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let signers: Vec<[u8; 32]> = keypairs_2.clone();


    let new_operators: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), keypairs_1),
        new_wghts: vec![&env, 1, 1],
        new_thres: 2,
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // approveContractCall converted into Bytes,
        params: vec![&env, new_operators.clone().to_xdr(&env)],
    };


    let proof: Validate = generate_test_proof(env.clone(), data.clone(), keypairs_2.clone(), weights, THRESHOLD, signers.clone());

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env),
    };

    // Initalize with 2 new random operators generated in proof
    let params_operator: Operatorship = Operatorship {
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));


    // Transfer operatorship to 2 new operators in the variable new_operators.
    // However, this should fail as the new operator's weights dont meet threshold.
    let test = input.to_xdr(&env);
    client.execute(&test);
}

// 'should not allow transferring operatorship with invalid threshold'
// this case differs by having a 0 threshold.
#[test]
#[should_panic]
fn invalid_threshold_3() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 2;
    const THRESHOLD: u128 = 0;
    let weights: Vec<u128> = vec![&env, 1, 1];

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);

    // Test Initalize
    let params_operator: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), keypairs.clone()),
        new_wghts: weights,
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));
}

// 'reject the proof if weights are not matching the threshold'
#[test]
#[should_panic]
fn low_signatures_weight() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 3;
    const THRESHOLD: u128 = 3;
    let weights: Vec<u128> = vec![&env, 1, 1, 1];

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let mut signers: Vec<[u8; 32]> = keypairs.clone();
    signers.remove(2); // signers no longer have enough weight to pass the threshold.

    let new_operators: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), keypairs.clone()),
        new_wghts: weights.clone(),
        new_thres: THRESHOLD,
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // approveContractCall converted into Bytes,
        params: vec![&env, new_operators.clone().to_xdr(&env)],
    };

    let proof: Validate = generate_test_proof(env.clone(), data.clone(), keypairs.clone(), weights, THRESHOLD, signers.clone());

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env),
    };

    // Initalize with 3 random operators
    let params_operator: Operatorship = Operatorship {
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));

    // As the signers do not have enough weight to pass the threshold, the proof check in execute() will error.
    let test = input.to_xdr(&env);
    client.execute(&test);
}

#[test]
#[should_panic]
fn invalid_commands() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const THRESHOLD: u128 = 3;
    let weights: Vec<u128> = vec![&env, 1, 1, 1];
    const NUM_OPS: u32 = 3;

    // Data for Contract Approve
    let params_approve = ContractPayload {
        src_chain: String::from_str(&env, "ethereum"),
        src_addr: String::from_str(&env, "0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d"),
        contract: String::from_str(&env, "0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d"),
        payload_ha: bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_tx_ha: bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_evnt: 1, // source event index // do u256 instead?
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0x0000000000000000000000000000000000000000000000000000000000000000)],
        commands: vec![&env, bytes![&env, 0x0000000000000000000000000000000000000000000000000000000000000000], bytes![&env, 0x0000000000000000000000000000000000000000000000000000000000000000]],
        params: vec![&env, params_approve.clone().to_xdr(&env)],
    };

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let signers: Vec<[u8; 32]> = keypairs.clone();
    let proof: Validate = generate_test_proof(env.clone(), data.clone(), keypairs.clone(), weights, THRESHOLD, signers.clone());


    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env),
    };

    // Initalize
    let params_operator: Operatorship = Operatorship {
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);
    client.initialize(&admin, &params_operator.clone().to_xdr(&env));

    let test = input.to_xdr(&env);
    client.execute(&test);
}

// 'reject the proof if signatures are invalid'
//#[test]
//#[should_panic]
fn invalid_signers() {
    let secp = Secp256k1::new();
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 3;
    const THRESHOLD: u128 = 3;
    let weights: Vec<u128> = vec![&env, 1, 1, 1];

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    // signers below are different from the operator keypairs above, causing the signature verification to fail successfully,
    let signers: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);

    let new_operators: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), keypairs.clone()),
        new_wghts: weights.clone(),
        new_thres: THRESHOLD,
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // approveContractCall converted into Bytes,
        params: vec![&env, new_operators.clone().to_xdr(&env)],
    };

    let secret_keys = generate_sorted_secret_keys(env.clone(), 2);
    let operator = &keypairs.get(0).unwrap();
    let incorrect_signer = &keypairs.get(1).unwrap();

    // The signature in the proof below does not match the operator. Therefore, this test case panics sucessfully.
    let proof: Validate = Validate {
        operators: vec![&env, generate_public_and_signature_key(env.clone(), data.clone(), operator).0],
        weights: vec![&env, THRESHOLD],
        threshold: THRESHOLD,
        signatures: vec![&env, generate_public_and_signature_key(env.clone(), data.clone(), incorrect_signer).1],
    };

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env),
    };

    // Initalize with 3 random operators
    let params_operator: Operatorship = Operatorship {
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));

    // As the signers do not have enough weight to pass the threshold, the proof check in execute() will error.
    let test = input.to_xdr(&env);
    client.execute(&test);
}

// 'reject the proof from the operators older than key retention'
#[test]
#[should_panic]
fn old_operators() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 3;
    const THRESHOLD: u128 = 3;
    let weights: Vec<u128> = vec![&env, 1, 1, 1];


    let init_keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let init_signers: Vec<[u8; 32]> = init_keypairs.clone();

    let mut prev_keypairs: Vec<[u8; 32]> = init_keypairs.clone();
    let mut prev_signers: Vec<[u8; 32]> = init_signers.clone();

    let initialize: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), init_keypairs.clone()),
        new_wghts: weights.clone(),
        new_thres: THRESHOLD,
    };

    let admin: Address = Address::generate(&env);
    client.initialize(&admin, &initialize.clone().to_xdr(&env));

    for i in 0..17 {
        env.budget().reset_default();
        let new_keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
        let new_signers: Vec<[u8; 32]> = new_keypairs.clone();

        let new_operators: Operatorship = Operatorship {
            new_ops: generate_mock_public_keys(env.clone(), new_keypairs.clone()),
            new_wghts: weights.clone(),
            new_thres: THRESHOLD,
        };

        let data: Data = Data {
            chain_id: 1,
            commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
            commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // transferOperatorship converted into Bytes,
            params: vec![&env, new_operators.clone().to_xdr(&env)],
        };

        // after the 17th operator, switch the proof back to using the initial keypairs so the proof fails from epoch - operators_epoch >= 16
        let proof: Validate;
        if i >= 16 {
            proof = generate_test_proof(env.clone(), data.clone(), init_keypairs.clone(), weights.clone(), THRESHOLD, init_signers.clone());
        } else {
            proof = generate_test_proof(env.clone(), data.clone(), prev_keypairs.clone(), weights.clone(), THRESHOLD, prev_signers.clone());
        }

        let input: Input = Input {
            data: data.clone(),
            proof: proof.clone().to_xdr(&env),
        };

        // Transfer operatorship to new operators in the variable new_operators
        let test = input.to_xdr(&env);
        client.execute(&test);

        prev_keypairs = new_keypairs.clone();
        prev_signers = new_signers.clone();
    }
}

// 'should not allow operatorship transfer to the previous operators '
#[test]
#[should_panic]
fn previous_operator() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const NUM_OPS: u32 = 3;
    const THRESHOLD: u128 = 3;
    let weights: Vec<u128> = vec![&env, 1, 1, 1];


    let init_keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let init_signers: Vec<[u8; 32]> = init_keypairs.clone();

    let mut prev_keypairs: Vec<[u8; 32]> = init_keypairs.clone();
    let mut prev_signers: Vec<[u8; 32]> = init_signers.clone();

    let initialize: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), init_keypairs.clone()),
        new_wghts: weights.clone(),
        new_thres: THRESHOLD,
    };

    let admin: Address = Address::generate(&env);
    client.initialize(&admin, &initialize.clone().to_xdr(&env));

    for i in 0..3 {
        env.budget().reset_default();
        let mut new_keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
        let mut new_signers: Vec<[u8; 32]> = new_keypairs.clone();

        // on second iteration, try transfering to the first operator.
        if i == 2 {
            new_keypairs = init_keypairs.clone();
            new_signers = init_keypairs.clone();
        }

        let new_operators: Operatorship = Operatorship {
            new_ops: generate_mock_public_keys(env.clone(), new_keypairs.clone()),
            new_wghts: weights.clone(),
            new_thres: THRESHOLD,
        };

        let data: Data = Data {
            chain_id: 1,
            commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
            commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // transferOperatorship converted into Bytes,
            params: vec![&env, new_operators.clone().to_xdr(&env)],
        };

        let proof = generate_test_proof(env.clone(), data.clone(), prev_keypairs.clone(), weights.clone(), THRESHOLD, prev_signers.clone());

        let input: Input = Input {
            data: data.clone(),
            proof: proof.clone().to_xdr(&env),
        };

        // Transfer operatorship to new operators in the variable new_operators
        let test = input.to_xdr(&env);
        client.execute(&test);

        prev_keypairs = new_keypairs.clone();
        prev_signers = new_signers.clone();
    }
}

// HELPER FUNCTIONS

fn generate_sorted_secret_keys(env: Env, num_ops: u32) -> Vec<[u8; 32]> {
    let mut operators: Vec<[u8; 32]> = Vec::new(&env);

    if num_ops == 0 {
        return operators;
    }

    let mut csprng = OsRng {};
    let secp = Secp256k1::new();

    for i in 0..num_ops {
        operators.push_back(SecretKey::new(&mut OsRng).secret_bytes());
    }

    for i in 0..operators.len() {
        for j in i + 1..operators.len() {
            let a = operators.get(i).unwrap();
            let b = operators.get(j).unwrap();
            if a > b {
                operators.set(i, b);
                operators.set(j, a);
            }
        }
    }

    return operators;
}

// 'should not allow transferring operatorship to address zero'
#[test]
#[should_panic]
fn transfer_zero() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    let new_operators: Operatorship = Operatorship {
        new_ops: vec![&env,
                      bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000),
                      bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002),
        ],
        new_wghts: vec![&env, 1, 1],
        new_thres: 2,
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // transferOperatorship converted into Bytes,
        params: vec![&env, new_operators.clone().to_xdr(&env)],
    };

    const THRESHOLD: u128 = 4;
    const NUM_OPS: u32 = 3;

    let keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);
    let mut signers: Vec<[u8; 32]> = keypairs.clone();
    signers.remove(1);

    let proof: Validate = generate_test_proof(env.clone(), data.clone(), keypairs.clone(), vec![&env, 1, 1, 3], THRESHOLD, signers.clone());

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env),
    };

    // Initalize with 3 random operators
    let params_operator: Operatorship = Operatorship {
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD,
    };
    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &params_operator.clone().to_xdr(&env));


    // Transfer operatorship to 2 new operators in the variable new_operators
    let test = input.to_xdr(&env);
    client.execute(&test);
}

// 'should expose correct hashes and epoch'
//#[test]
fn hashForEpoch_epochForHash() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    const THRESHOLD: u128 = 3;
    let weights: Vec<u128> = vec![&env, 1, 1, 1];
    const NUM_OPS: u32 = 3;
    //let weights: Vec<u128> = vec![&env, 1, 1, 1];

    let init_keypairs: Vec<[u8; 32]> = generate_sorted_secret_keys(env.clone(), NUM_OPS);

    let init_operators: Operatorship = Operatorship {
        new_ops: generate_mock_public_keys(env.clone(), init_keypairs.clone()),
        new_wghts: weights.clone(),
        new_thres: THRESHOLD,
    };

    let admin: Address = Address::generate(&env);

    client.initialize(&admin, &init_operators.clone().to_xdr(&env));

    let epoch: u128 = env.as_contract(&contract_id, || env.storage().instance().get(&Symbol::new(&env, "current_epoch")).unwrap_or(0));
    let new_operators_hash: BytesN<32> = env.crypto().keccak256(&init_operators.to_xdr(&env));
    let new_operators_hash_key: BytesN<32> = env.crypto().keccak256(&PrefixHash { prefix: Symbol::new(&env, "operators_for_epoch"), hash: new_operators_hash.clone() }.to_xdr(&env));

    let hash_for_epoch: u128 = env.as_contract(&contract_id, || env.storage().instance().get(&new_operators_hash_key).unwrap_or(0));
    let epoch_for_hash: BytesN<32> = env.as_contract(&contract_id, || env.storage().instance().get(&PrefixEpoch { prefix: Symbol::new(&env, "epoch_for_operators"), epoch }).unwrap());

    assert_eq!(hash_for_epoch, epoch);
    assert_eq!(epoch_for_hash, new_operators_hash);
}

// signers is a subset of operators that is signing the data
// only signers with biggest weight to pass need to sign it.
// ASSUMPTION: operators & signers are ordered.
fn generate_test_proof(
    env: Env,
    data: Data,
    operators: Vec<[u8; 32]>,
    weights: Vec<u128>,
    threshold: u128,
    signers: Vec<[u8; 32]>,
) -> Validate {
    let secp = Secp256k1::new();
    // Create signatures & weights
    let mut signatures: Vec<BytesN<65>> = Vec::new(&env);

    // now looping through signers
    for i in 0..signers.len() {
        // NEXT: want to find index of the signers inside of operators.
        // THEN, add the signature & that index to signature.
        let mut operator_index: u32 = u32::MAX; // is there a potential security exploit doing it this way?
        for index in 0..operators.len() {
            if signers.get(i).unwrap() == operators.get(index).unwrap() {
                operator_index = index;
                break;
            }
        }

        // signers is not a subset of operators.
        if operator_index == u32::MAX {
            panic!();
        }

        let secret_key = &SecretKey::from_slice(
            &signers.get(i).unwrap()
        ).unwrap();

        let hash: BytesN<32> = env.crypto().keccak256(&data.clone().to_xdr(&env));
        let signed_message_hash: BytesN<32> = Gateway::to_signed_msg_hsh(env.clone(), hash);
        let message = Message::from_digest_slice(&signed_message_hash.to_array()).unwrap();

        let signature = secp.sign_ecdsa_recoverable(&message, secret_key);
        let (recid, encoded_sig) = signature.serialize_compact();
        let encoded_recid = u8::try_from(recid.to_i32() + 27).unwrap().to_le_bytes();
        let encoded_recoverable_sig: [u8; 65] = [&encoded_sig[..], &encoded_recid[..]].concat().try_into().unwrap();
        let signature_bytes: BytesN<65> = BytesN::from_array(&env, &encoded_recoverable_sig);

        signatures.push_back(signature_bytes);
    }

    // Create operators
    // let mut operators_pk: Vec<BytesN<65>> = &operators.map(|x| BytesN::from_array(&env, &x.to_public_key(&secp).serialize_compact()));

    let mut operators_pk: Vec<BytesN<65>> = Vec::new(&env);

    for i in 0..operators.len() {
        operators_pk.push_back(
            BytesN::from_array(
                &env,
                &SecretKey::from_slice(
                    &operators.get(i).unwrap()
                ).unwrap().public_key(&secp).serialize_uncompressed(),
            ));
    }


    let proof: Validate = Validate {
        operators: operators_pk,
        weights,
        threshold, // uint256
        signatures,
    };

    proof
}

fn generate_public_and_signature_key(env: Env, data: Data, secret_key_bytes: &[u8; 32]) -> (BytesN<65>, BytesN<65>) {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(secret_key_bytes).unwrap();

    let hash: BytesN<32> = env.crypto().keccak256(&data.clone().to_xdr(&env));
    let signed_message_hash: BytesN<32> = Gateway::to_signed_msg_hsh(env.clone(), hash);
    let message = Message::from_digest_slice(&signed_message_hash.to_array()).unwrap();


    let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);
    let (recid, encoded_sig) = signature.serialize_compact();
    let encoded_recid = u8::try_from(recid.to_i32() + 27).unwrap().to_le_bytes();
    let encoded_recoverable_sig: [u8; 65] = [&encoded_sig[..], &encoded_recid[..]].concat().try_into().unwrap();
    let signature_bytes: BytesN<65> = BytesN::from_array(&env, &encoded_recoverable_sig);

    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    let public_key_bytes: BytesN<65> = BytesN::from_array(&env, &public_key.serialize_uncompressed());

    return (public_key_bytes, signature_bytes);
}


fn generate_mock_public_keys(env: Env, secret_keys: Vec<[u8; 32]>) -> Vec<BytesN<65>> {
    let secp = Secp256k1::new();
    let mut operators: Vec<BytesN<65>> = Vec::new(&env);

    for i in 0..secret_keys.len() {
        operators.push_back(BytesN::from_array(&env, &PublicKey::from_secret_key(
            &secp,
            &SecretKey::from_slice(
                &secret_keys.get(i).unwrap()
            ).unwrap(),
        ).serialize_uncompressed()));
    }

    return operators;
}