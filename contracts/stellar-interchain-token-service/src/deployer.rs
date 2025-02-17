use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, BytesN, Env};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_std::events::Event;

use crate::event::{InterchainTokenDeployedEvent, TokenManagerDeployedEvent};
use crate::types::TokenManagerType;

/// This prefix along with the tokenId is used to generate the salt for the deterministic interchain token deployment
const PREFIX_INTERCHAIN_TOKEN_DEPLOYMENT_SALT: &str = "its-interchain-token-salt";
/// This prefix, along with the tokenId, is used to generate the salt for the deterministic token manager deployment
const PREFIX_TOKEN_MANAGER_DEPLOYMENT_SALT: &str = "its-token-manager-salt";

fn interchain_token_deployment_salt(env: &Env, token_id: BytesN<32>) -> BytesN<32> {
    env.crypto()
        .keccak256(&(PREFIX_INTERCHAIN_TOKEN_DEPLOYMENT_SALT, token_id).to_xdr(env))
        .into()
}

pub fn interchain_token_address(env: &Env, token_id: BytesN<32>) -> Address {
    env.deployer()
        .with_current_contract(interchain_token_deployment_salt(env, token_id))
        .deployed_address()
}

fn token_manager_deployment_salt(env: &Env, token_id: BytesN<32>) -> BytesN<32> {
    env.crypto()
        .keccak256(&(PREFIX_TOKEN_MANAGER_DEPLOYMENT_SALT, token_id).to_xdr(env))
        .into()
}

pub fn token_manager_address(env: &Env, token_id: BytesN<32>) -> Address {
    env.deployer()
        .with_current_contract(token_manager_deployment_salt(env, token_id))
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
        .with_current_contract(interchain_token_deployment_salt(env, token_id.clone()))
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
        .with_current_contract(token_manager_deployment_salt(env, token_id.clone()))
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
