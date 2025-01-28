use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, BytesN, Env};
use stellar_axelar_std::address::AddressExt;

const PREFIX_CANONICAL_TOKEN_SALT: &str = "canonical-token-salt";
const PREFIX_INTERCHAIN_TOKEN_SALT: &str = "interchain-token-salt";
/// This prefix is used along with a salt to generate the token ID
const PREFIX_TOKEN_ID: &str = "its-interchain-token-id";

fn token_id(env: &Env, deploy_salt: BytesN<32>) -> BytesN<32> {
    env.crypto()
        .keccak256(&(PREFIX_TOKEN_ID, Address::zero(env), deploy_salt).to_xdr(env))
        .into()
}

fn canonical_token_deploy_salt(
    env: &Env,
    chain_name_hash: BytesN<32>,
    token_address: Address,
) -> BytesN<32> {
    env.crypto()
        .keccak256(&(PREFIX_CANONICAL_TOKEN_SALT, chain_name_hash, token_address).to_xdr(env))
        .into()
}

pub fn canonical_interchain_token_id(
    env: &Env,
    chain_name_hash: BytesN<32>,
    token_address: Address,
) -> BytesN<32> {
    token_id(
        env,
        canonical_token_deploy_salt(env, chain_name_hash, token_address),
    )
}

fn interchain_token_deploy_salt(
    env: &Env,
    chain_name_hash: BytesN<32>,
    deployer: Address,
    salt: BytesN<32>,
) -> BytesN<32> {
    env.crypto()
        .keccak256(
            &(
                PREFIX_INTERCHAIN_TOKEN_SALT,
                chain_name_hash,
                deployer,
                salt,
            )
                .to_xdr(env),
        )
        .into()
}

pub fn interchain_token_id(
    env: &Env,
    chain_name_hash: BytesN<32>,
    deployer: Address,
    salt: BytesN<32>,
) -> BytesN<32> {
    token_id(
        env,
        interchain_token_deploy_salt(env, chain_name_hash, deployer, salt),
    )
}
