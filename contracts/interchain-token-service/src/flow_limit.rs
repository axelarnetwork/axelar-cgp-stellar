use axelar_soroban_std::{ensure, events::Event, ttl::extend_persistent_ttl};
use soroban_sdk::{BytesN, Env};

use crate::{
    error::ContractError,
    event::FlowLimitSetEvent,
    storage_types::{DataKey, FlowKey},
};

const EPOCH_TIME: u64 = 6 * 60 * 60; // 6 hours in seconds = 21600

pub enum FlowDirection {
    /// An interchain transfer coming in to this chain from another chain
    In,
    /// An interchain transfer going out from this chain to another chain
    Out,
}

impl FlowDirection {
    fn flow(&self, env: &Env, token_id: BytesN<32>) -> i128 {
        match self {
            Self::In => flow_in_amount(env, token_id),
            Self::Out => flow_out_amount(env, token_id),
        }
    }

    fn reverse_flow(&self, env: &Env, token_id: BytesN<32>) -> i128 {
        match self {
            Self::In => flow_out_amount(env, token_id),
            Self::Out => flow_in_amount(env, token_id),
        }
    }

    fn update_flow(&self, env: &Env, token_id: BytesN<32>, new_flow: i128) {
        let flow_key = FlowKey {
            token_id,
            epoch: current_epoch(env),
        };

        let key = match self {
            Self::In => DataKey::FlowIn(flow_key),
            Self::Out => DataKey::FlowOut(flow_key),
        };

        env.storage().temporary().set(&key, &new_flow);
    }

    /// Adds flow amount in the specified direction (in/out) for a token.
    /// Flow amounts are stored in temporary storage since they only need to persist for
    /// the 6-hour epoch duration.
    ///
    /// Checks that:
    /// - Flow amount doesn't exceed the flow limit
    /// - Adding flows won't cause overflow
    /// - Net flow (outgoing minus incoming flow) doesn't exceed the limit
    pub fn add_flow(
        &self,
        env: &Env,
        token_id: BytesN<32>,
        flow_amount: i128,
    ) -> Result<(), ContractError> {
        let Some(flow_limit) = flow_limit(env, token_id.clone()) else {
            return Ok(());
        };

        ensure!(flow_amount <= flow_limit, ContractError::FlowLimitExceeded);

        let new_flow = self
            .flow(env, token_id.clone())
            .checked_add(flow_amount)
            .ok_or(ContractError::FlowAmountOverflow)?;
        let max_allowed = self
            .reverse_flow(env, token_id.clone())
            .checked_add(flow_limit)
            .ok_or(ContractError::FlowAmountOverflow)?;

        // Equivalent to flow_amount + flow - reverse_flow <= flow_limit
        ensure!(new_flow <= max_allowed, ContractError::FlowLimitExceeded);

        self.update_flow(env, token_id.clone(), new_flow);

        extend_persistent_ttl(env, &DataKey::FlowLimit(token_id));

        Ok(())
    }
}

fn current_epoch(env: &Env) -> u64 {
    env.ledger().timestamp() / EPOCH_TIME
}

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
        ensure!(limit >= 0, ContractError::InvalidFlowLimit);
    }

    if let Some(limit) = flow_limit {
        env.storage()
            .persistent()
            .set(&DataKey::FlowLimit(token_id.clone()), &limit);
    } else {
        env.storage()
            .persistent()
            .remove(&DataKey::FlowLimit(token_id.clone()));
    }

    FlowLimitSetEvent {
        token_id,
        flow_limit,
    }
    .emit(env);

    Ok(())
}

pub fn flow_out_amount(env: &Env, token_id: BytesN<32>) -> i128 {
    env.storage()
        .temporary()
        .get(&DataKey::FlowOut(FlowKey {
            token_id,
            epoch: current_epoch(env),
        }))
        .unwrap_or(0)
}

pub fn flow_in_amount(env: &Env, token_id: BytesN<32>) -> i128 {
    env.storage()
        .temporary()
        .get(&DataKey::FlowIn(FlowKey {
            token_id,
            epoch: current_epoch(env),
        }))
        .unwrap_or(0)
}
