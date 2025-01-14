use axelar_gateway::executable::AxelarExecutableInterface;
use axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;

use crate::{error::ContractError, types::TokenManagerType};

#[allow(dead_code)]
#[contractclient(name = "InterchainTokenServiceClient")]
pub trait InterchainTokenServiceInterface: AxelarExecutableInterface {
    fn chain_name(env: &Env) -> String;

    fn gas_service(env: &Env) -> Address;

    fn interchain_token_wasm_hash(env: &Env) -> BytesN<32>;

    fn its_hub_address(env: &Env) -> String;

    fn its_hub_chain_name(env: &Env) -> String;

    fn is_trusted_chain(env: &Env, chain: String) -> bool;

    fn set_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError>;

    fn remove_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError>;

    fn interchain_token_deploy_salt(env: &Env, deployer: Address, salt: BytesN<32>) -> BytesN<32>;

    fn interchain_token_id(env: &Env, sender: Address, salt: BytesN<32>) -> BytesN<32>;

    fn canonical_token_deploy_salt(env: &Env, token_address: Address) -> BytesN<32>;

    fn token_address(env: &Env, token_id: BytesN<32>) -> Address;

    fn token_manager_type(env: &Env, token_id: BytesN<32>) -> TokenManagerType;

    /// Retrieves the flow limit for the token associated with the specified token ID.
    /// Returns `None` if no limit is set.
    fn flow_limit(env: &Env, token_id: BytesN<32>) -> Option<i128>;

    /// Retrieves the amount that has flowed out of the chain to other chains during the current epoch
    /// for the token associated with the specified token ID.
    fn flow_out_amount(env: &Env, token_id: BytesN<32>) -> i128;

    /// Retrieves the amount that has flowed into the chain from other chains during the current epoch
    /// for the token associated with the specified token ID.
    fn flow_in_amount(env: &Env, token_id: BytesN<32>) -> i128;

    /// Sets or updates the flow limit for a token.
    ///
    /// Flow limit controls how many tokens can flow in/out during a single epoch.
    /// Setting the limit to `None` disables flow limit checks for the token.
    /// Setting the limit to 0 effectively freezes the token by preventing any flow.
    ///
    /// # Arguments
    /// - `token_id`: Unique identifier of the token.
    /// - `flow_limit`: The new flow limit value. Must be positive if Some.
    ///
    /// # Returns
    /// - `Result<(), ContractError>`: Ok(()) on success.
    ///
    /// # Errors
    /// - `ContractError::InvalidFlowLimit`: If the provided flow limit is not positive.
    ///
    /// # Authorization
    /// - Must be called by the [`Self::operator`].
    fn set_flow_limit(
        env: &Env,
        token_id: BytesN<32>,
        flow_limit: Option<i128>,
    ) -> Result<(), ContractError>;

    fn deploy_interchain_token(
        env: &Env,
        deployer: Address,
        salt: BytesN<32>,
        token_metadata: TokenMetadata,
        initial_supply: i128,
        minter: Option<Address>,
    ) -> Result<BytesN<32>, ContractError>;

    fn deploy_remote_interchain_token(
        env: &Env,
        caller: Address,
        salt: BytesN<32>,
        destination_chain: String,
        gas_token: Token,
    ) -> Result<BytesN<32>, ContractError>;

    fn deploy_remote_canonical_token(
        env: &Env,
        token_address: Address,
        destination_chain: String,
        spender: Address,
        gas_token: Token,
    ) -> Result<BytesN<32>, ContractError>;

    fn interchain_transfer(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        destination_address: Bytes,
        amount: i128,
        metadata: Option<Bytes>,
        gas_token: Token,
    ) -> Result<(), ContractError>;

    fn register_canonical_token(
        env: &Env,
        token_address: Address,
    ) -> Result<BytesN<32>, ContractError>;
}
