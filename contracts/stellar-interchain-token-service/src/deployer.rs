use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, BytesN, Env};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_std::events::Event;

use crate::event::{InterchainTokenDeployedEvent, TokenManagerDeployedEvent};
use crate::types::TokenManagerType;

const PREFIX_INTERCHAIN_TOKEN_ID: &str = "its-interchain-token-id";
const PREFIX_CANONICAL_TOKEN_SALT: &str = "canonical-token-salt";
const PREFIX_INTERCHAIN_TOKEN_SALT: &str = "its-interchain-token-salt";
const PREFIX_TOKEN_MANAGER: &str = "its-token-manager-salt";

pub fn interchain_token_id(env: &Env, sender: Address, salt: BytesN<32>) -> BytesN<32> {
    env.crypto()
        .keccak256(&(PREFIX_INTERCHAIN_TOKEN_ID, sender, salt).to_xdr(env))
        .into()
}

fn interchain_token_salt(env: &Env, token_id: BytesN<32>) -> BytesN<32> {
    env.crypto()
        .keccak256(&(PREFIX_INTERCHAIN_TOKEN_SALT, token_id).to_xdr(env))
        .into()
}

pub fn interchain_token_address(env: &Env, token_id: BytesN<32>) -> Address {
    env.deployer()
        .with_current_contract(interchain_token_salt(env, token_id.clone()))
        .deployed_address()
}

fn token_manager_salt(env: &Env, token_id: BytesN<32>) -> BytesN<32> {
    env.crypto()
        .keccak256(&(PREFIX_TOKEN_MANAGER, token_id).to_xdr(env))
        .into()
}

pub fn token_manager_address(env: &Env, token_id: BytesN<32>) -> Address {
    env.deployer()
        .with_current_contract(token_manager_salt(env, token_id.clone()))
        .deployed_address()
}

pub fn deploy_interchain_token(
    env: &Env,
    interchain_token_wasm_hash: BytesN<32>,
    minter: Option<Address>,
    token_id: BytesN<32>,
    token_metadata: TokenMetadata,
) -> Address {
    let deployed_address = env
        .deployer()
        .with_address(
            env.current_contract_address(),
            interchain_token_salt(env, token_id.clone()),
        )
        .deploy_v2(
            interchain_token_wasm_hash,
            (
                env.current_contract_address(),
                minter.clone(),
                token_id.clone(),
                token_metadata.clone(),
            ),
        );

    InterchainTokenDeployedEvent {
        token_id,
        token_address: deployed_address.clone(),
        name: token_metadata.name,
        symbol: token_metadata.symbol,
        decimals: token_metadata.decimal,
        minter,
    }
    .emit(env);

    deployed_address
}

pub fn deploy_token_manager(
    env: &Env,
    token_manager_wasm_hash: BytesN<32>,
    token_id: BytesN<32>,
    token_address: Address,
    token_manager_type: TokenManagerType,
) -> Address {
    let deployed_address = env
        .deployer()
        .with_address(
            env.current_contract_address(),
            token_manager_salt(env, token_id.clone()),
        )
        .deploy_v2(token_manager_wasm_hash, (env.current_contract_address(),));

    TokenManagerDeployedEvent {
        token_id,
        token_address,
        token_manager: deployed_address.clone(),
        token_manager_type,
    }
    .emit(env);

    deployed_address
}
