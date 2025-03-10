use soroban_sdk::{vec, BytesN, String};
use stellar_axelar_std::interfaces::CustomMigratableInterface;
use stellar_axelar_std::{assert_auth, assert_err};

use crate::contract::AxelarGateway;
use crate::error::ContractError;
use crate::storage::MessageApprovalValue;
use crate::tests::testutils::{
    get_message_approval, setup_env, setup_legacy_message_approval, TestConfig,
};

const NEW_WASM: &[u8] = include_bytes!("testdata/stellar_axelar_gateway.optimized.wasm");

#[test]
fn migrate_succeeds_with_valid_message_approvals() {
    let TestConfig { env, client, .. } = setup_env(1, 5);

    let owner = client.owner();

    let new_wasm_hash = env.deployer().upload_contract_wasm(NEW_WASM);

    let source_chain1 = String::from_str(&env, "ethereum");
    let message_id1 = String::from_str(&env, "message1");
    let hash1: BytesN<32> = BytesN::from_array(&env, &[1; 32]);

    let source_chain2 = String::from_str(&env, "polygon");
    let message_id2 = String::from_str(&env, "message2");

    env.as_contract(&client.address, || {
        setup_legacy_message_approval(
            &env,
            source_chain1.clone(),
            message_id1.clone(),
            MessageApprovalValue::Approved(hash1.clone()),
        );
        setup_legacy_message_approval(
            &env,
            source_chain2.clone(),
            message_id2.clone(),
            MessageApprovalValue::Executed,
        );
    });

    assert_auth!(owner, client.upgrade(&new_wasm_hash));

    let migration_data = vec![
        &env,
        (source_chain1.clone(), message_id1.clone()),
        (source_chain2.clone(), message_id2.clone()),
    ];

    assert_auth!(owner, client.migrate(&migration_data));

    assert_eq!(
        get_message_approval(&env, &client.address, &source_chain1, &message_id1),
        MessageApprovalValue::Approved(hash1)
    );
    assert_eq!(
        get_message_approval(&env, &client.address, &source_chain2, &message_id2),
        MessageApprovalValue::Executed
    );
}

#[test]
fn migrate_fails_when_message_approval_not_found() {
    let TestConfig { env, client, .. } = setup_env(1, 5);

    let owner = client.owner();

    let new_wasm_hash = env.deployer().upload_contract_wasm(NEW_WASM);

    let source_chain1 = String::from_str(&env, "ethereum");
    let message_id1 = String::from_str(&env, "message1");
    let hash1: BytesN<32> = BytesN::from_array(&env, &[1; 32]);

    let source_chain2 = String::from_str(&env, "polygon");
    let message_id2 = String::from_str(&env, "non_existent");

    env.as_contract(&client.address, || {
        setup_legacy_message_approval(
            &env,
            source_chain1.clone(),
            message_id1.clone(),
            MessageApprovalValue::Approved(hash1.clone()),
        );
    });

    assert_auth!(owner, client.upgrade(&new_wasm_hash));

    let migration_data = vec![
        &env,
        (source_chain1, message_id1),
        (source_chain2, message_id2),
    ];

    assert_err!(
        env.as_contract(&client.address, || {
            <AxelarGateway as CustomMigratableInterface>::__migrate(&env, migration_data)
        }),
        ContractError::MessageApprovalNotFound
    );
}

#[test]
fn migrate_succeeds_with_empty_migration_data() {
    let TestConfig { env, client, .. } = setup_env(1, 5);

    let owner = client.owner();

    let new_wasm_hash = env.deployer().upload_contract_wasm(NEW_WASM);

    assert_auth!(owner, client.upgrade(&new_wasm_hash));

    let migration_data = vec![&env];

    assert_auth!(owner, client.migrate(&migration_data));
}
