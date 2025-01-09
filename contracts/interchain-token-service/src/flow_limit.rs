use axelar_soroban_std::ensure;
use axelar_soroban_std::events::Event;
use axelar_soroban_std::ttl::extend_persistent_ttl;
use soroban_sdk::{BytesN, Env};

use crate::error::ContractError;
use crate::event::FlowLimitSetEvent;
use crate::storage_types::DataKey;

const EPOCH_TIME: u64 = 6 * 60 * 60; // 6 hours in seconds = 21600

pub fn flow_limit(env: &Env, token_id: BytesN<32>) -> Option<i128> {
    env.storage()
        .persistent()
        .get(&DataKey::FlowLimit(token_id))
}

pub fn set_flow_limit(
    env: &Env,
    token_id: BytesN<32>,
    flow_limit: Option<i128>,
) -> Result<(), ContractError> {
    if let Some(limit) = flow_limit {
        ensure!(limit > 0, ContractError::InvalidFlowLimit);
    }

    env.storage()
        .persistent()
        .set(&DataKey::FlowLimit(token_id.clone()), &flow_limit);

    FlowLimitSetEvent {
        token_id,
        flow_limit,
    }
    .emit(env);

    Ok(())
}

pub fn flow_out_amount(env: &Env, token_id: BytesN<32>) -> i128 {
    let epoch = env.ledger().timestamp() / EPOCH_TIME;
    env.storage()
        .temporary()
        .get(&DataKey::FlowOut(token_id, epoch))
        .unwrap_or(0)
}

pub fn flow_in_amount(env: &Env, token_id: BytesN<32>) -> i128 {
    let epoch = env.ledger().timestamp() / EPOCH_TIME;
    env.storage()
        .temporary()
        .get(&DataKey::FlowIn(token_id, epoch))
        .unwrap_or(0)
}

enum FlowDirection {
    In,
    Out,
}

fn add_flow(
    env: &Env,
    token_id: BytesN<32>,
    flow_amount: i128,
    direction: FlowDirection,
) -> Result<(), ContractError> {
    let Some(flow_limit) = flow_limit(env, token_id.clone()) else {
        return Ok(());
    };

    let epoch = env.ledger().timestamp() / EPOCH_TIME;

    let (flow_to_add_key, flow_to_compare_key) = match direction {
        FlowDirection::In => (
            DataKey::FlowIn(token_id.clone(), epoch),
            DataKey::FlowOut(token_id.clone(), epoch),
        ),
        FlowDirection::Out => (
            DataKey::FlowOut(token_id.clone(), epoch),
            DataKey::FlowIn(token_id.clone(), epoch),
        ),
    };

    let flow_to_add: i128 = env.storage().temporary().get(&flow_to_add_key).unwrap_or(0);
    let flow_to_compare: i128 = env
        .storage()
        .temporary()
        .get(&flow_to_compare_key)
        .unwrap_or(0);

    ensure!(flow_amount <= flow_limit, ContractError::FlowLimitExceeded);

    let new_flow = flow_to_add
        .checked_add(flow_amount)
        .ok_or(ContractError::FlowLimitExceeded)?;
    let max_allowed = flow_to_compare
        .checked_add(flow_limit)
        .ok_or(ContractError::FlowLimitExceeded)?;

    ensure!(new_flow <= max_allowed, ContractError::FlowLimitExceeded);

    env.storage().temporary().set(&flow_to_add_key, &new_flow);

    extend_persistent_ttl(env, &DataKey::FlowLimit(token_id));

    Ok(())
}

pub fn add_flow_in(
    env: &Env,
    token_id: BytesN<32>,
    flow_amount: i128,
) -> Result<(), ContractError> {
    add_flow(env, token_id, flow_amount, FlowDirection::In)
}

pub fn add_flow_out(
    env: &Env,
    token_id: BytesN<32>,
    flow_amount: i128,
) -> Result<(), ContractError> {
    add_flow(env, token_id, flow_amount, FlowDirection::Out)
}
