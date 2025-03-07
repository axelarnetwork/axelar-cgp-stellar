use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gateway::executable::AxelarExecutableInterface;
use stellar_axelar_std::interfaces::{
    OperatableInterface, OwnableInterface, PausableInterface, UpgradableInterface,
};
use stellar_axelar_std::types::Token;

use crate::error::ContractError;
use crate::types::TokenManagerType;

#[contractclient(name = "InterchainTokenServiceClient")]
pub trait InterchainTokenServiceInterface:
    AxelarExecutableInterface
    + OwnableInterface
    + OperatableInterface
    + PausableInterface
    + UpgradableInterface
{
    /// Returns the address of the Gas Service contract.
    fn gas_service(env: &Env) -> Address;

    /// Returns the name of the current chain.
    fn chain_name(env: &Env) -> String;

    /// Returns the name of the chain on which the ITS Hub is deployed.
    fn its_hub_chain_name(env: &Env) -> String;

    /// Returns the address of the ITS Hub.
    fn its_hub_address(env: &Env) -> String;

    /// Returns the address of the native token on the current chain.
    fn native_token_address(env: &Env) -> Address;

    /// Returns the WASM hash of the token contract used for deploying interchain tokens.
    fn interchain_token_wasm_hash(env: &Env) -> BytesN<32>;

    /// Returns the WASM hash of the token manager contract used for deploying token managers.
    fn token_manager_wasm_hash(env: &Env) -> BytesN<32>;

    /// Returns whether the specified chain is trusted for cross-chain messaging.
    fn is_trusted_chain(env: &Env, chain: String) -> bool;

    /// Sets the specified chain as trusted for cross-chain messaging.
    /// # Authorization
    /// - [`OwnableInterface::owner`] must authorize.
    fn set_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError>;

    /// Removes the specified chain from trusted chains.
    /// # Authorization
    /// - [`OwnableInterface::owner`] must authorize.
    fn remove_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError>;

    /// Computes the unique identifier for an interchain token.
    ///
    /// The token ID is derived uniquely from the deployer's address and the provided salt.
    ///
    /// # Parameters
    /// - `deployer`: The address of the token deployer. In the case of tokens deployed by this contract, it will be Stellar's "dead" address.
    /// - `salt`: A unique value used to generate the token ID.
    ///
    /// # Returns
    /// - A `BytesN<32>` value representing the token's unique ID.
    fn interchain_token_id(env: &Env, deployer: Address, salt: BytesN<32>) -> BytesN<32>;

    /// Computes a 32-byte deployment salt for a canonical token.
    ///
    /// The salt is derived uniquely from the chain name hash and token address.
    ///
    /// # Parameters
    /// - `token_address`: The address of the token for which the deployment salt is being generated.
    ///
    /// # Returns
    /// - A `BytesN<32>` value representing the computed deployment salt.
    fn canonical_interchain_token_id(env: &Env, token_address: Address) -> BytesN<32>;

    /// Returns the predicted address of the native interchain token associated with the specified token ID.
    ///
    /// # Arguments
    /// - `token_id`: The token ID for the interchain token.
    ///
    /// # Returns
    /// - `Address`: The address of the interchain token.
    fn interchain_token_address(env: &Env, token_id: BytesN<32>) -> Address;

    /// Returns the predicted address of the token manager that will be deployed for the specified token ID.
    ///
    /// # Arguments
    /// - `token_id`: The token ID for the interchain token.
    ///
    /// # Returns
    /// - `Address`: The address of the token manager.
    fn token_manager_address(env: &Env, token_id: BytesN<32>) -> Address;

    /// Returns the address of the token associated with the specified token ID.
    fn registered_token_address(env: &Env, token_id: BytesN<32>) -> Address;

    /// Returns the address of the token manager associated with the specified token ID.
    fn deployed_token_manager(env: &Env, token_id: BytesN<32>) -> Address;

    /// Returns the type of the token manager associated with the specified token ID.
    fn token_manager_type(env: &Env, token_id: BytesN<32>) -> TokenManagerType;

    /// Returns the flow limit for the token associated with the specified token ID.
    /// Returns `None` if no limit is set.
    fn flow_limit(env: &Env, token_id: BytesN<32>) -> Option<i128>;

    /// Returns the amount that has flowed out of the chain to other chains during the current epoch
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
    /// # Errors
    /// - [`ContractError::InvalidFlowLimit`]: If the provided flow limit is not positive.
    ///
    /// # Authorization
    /// - [`OperatableInterface::operator`] must authorize.
    fn set_flow_limit(
        env: &Env,
        token_id: BytesN<32>,
        flow_limit: Option<i128>,
    ) -> Result<(), ContractError>;

    /// Deploys a new interchain token on the current chain with specified metadata and optional
    /// initial supply. If initial supply is provided, it is minted to the caller. The
    /// caller can also specify an optional minter address for the interchain token.
    ///
    /// # Arguments
    /// - `caller`: Address of the caller initiating the deployment.
    /// - `salt`: A 32-byte unique salt used for token deployment.
    /// - `token_metadata`: Metadata for the new token (name, symbol, decimals).
    /// - `initial_supply`: Initial amount to mint to caller, if greater than 0.
    /// - `minter`: Optional address that will have a minter role for the deployed interchain token.
    ///
    /// # Returns
    /// - `Ok(BytesN<32>)`: Returns the token ID.
    ///
    /// # Errors
    /// - [`ContractError::InvalidMinter`]: If the minter address is invalid.
    ///
    /// # Authorization
    /// - The `deployer` must authorize.
    fn deploy_interchain_token(
        env: &Env,
        deployer: Address,
        salt: BytesN<32>,
        token_metadata: TokenMetadata,
        initial_supply: i128,
        minter: Option<Address>,
    ) -> Result<BytesN<32>, ContractError>;

    /// Initiates the deployment of an interchain token to a specified destination chain.
    ///
    /// # Arguments
    /// - `caller`: Address of the caller initiating the deployment.
    /// - `salt`: A 32-byte unique salt used for token deployment.
    /// - `destination_chain`: The name of the destination chain where the token will be deployed.
    /// - `gas_token`: An optional gas token used to pay for the gas cost of the cross-chain call.
    ///
    /// # Returns
    /// - `Ok(BytesN<32>)`: Returns the token ID.
    ///
    /// # Errors
    /// - [`ContractError::InvalidTokenId`]: If the token ID does not exist in the persistent storage.
    /// - Any error propagated from `pay_gas_and_call_contract`.
    ///
    /// # Authorization
    /// - The `caller` must authorize.
    fn deploy_remote_interchain_token(
        env: &Env,
        caller: Address,
        salt: BytesN<32>,
        destination_chain: String,
        gas_token: Option<Token>,
    ) -> Result<BytesN<32>, ContractError>;

    /// Registers a canonical token as an interchain token.
    ///
    /// # Arguments
    /// - `token_address` - The address of the canonical token.
    ///
    /// # Returns
    /// - `Ok(BytesN<32>)`: Returns the token ID.
    ///
    /// # Errors
    /// - [`ContractError::TokenAlreadyRegistered`]: If the token ID is already registered.
    fn register_canonical_token(
        env: &Env,
        token_address: Address,
    ) -> Result<BytesN<32>, ContractError>;

    /// Deploys a remote canonical token on a specified destination chain.
    ///
    /// Anyone can call this to deploy a trustless canonical representation of the token to any trusted destination chain.
    /// If the token name is longer than 32 characters, the symbol will be used as the name.
    /// Specifically, natively issued Stellar assets will be deployed with the symbol as the name.
    ///
    /// # Arguments
    /// * `token_address` - The address of the token to be deployed.
    /// * `destination_chain` - The name of the destination chain where the token will be deployed.
    /// * `spender` - The spender of the cross-chain gas.
    /// * `gas_token` - An optional gas token used to pay for gas during the deployment.
    ///
    /// # Returns
    /// - `Ok(BytesN<32>)`: Returns the token ID.
    ///
    /// # Errors
    /// - [`ContractError::InvalidTokenId`]: If the token ID does not exist in the persistent storage.
    /// - Any error propagated from `pay_gas_and_call_contract`.
    ///
    /// # Authorization
    /// - `spender` needs to authorize `pay_gas` call to the gas service.
    fn deploy_remote_canonical_token(
        env: &Env,
        token_address: Address,
        destination_chain: String,
        spender: Address,
        gas_token: Option<Token>,
    ) -> Result<BytesN<32>, ContractError>;

    /// Initiates a cross-chain token transfer.
    ///
    /// Takes tokens from the caller on the source chain and initiates a transfer
    /// to the specified destination chain. The tokens will be transferred to the destination address
    /// when the message is executed on the destination chain.
    ///
    /// If `data` is provided, the `destination_address` will also be executed as a contract with the
    /// `data` to allow arbitrary processing of the transferred tokens.
    ///
    /// # Arguments
    /// - `caller`: The address initiating the transfer.
    /// - `token_id`: The unique identifier of the token being transferred.
    /// - `destination_chain`: The chain to which tokens will be transferred.
    /// - `destination_address`: The recipient address on the destination chain.
    /// - `amount`: The amount of tokens to transfer. Must be greater than 0.
    /// - `data`: Optional data to be handled by the destination address if it's a contract.
    /// - `gas_token`: An optional gas token used to pay for cross-chain message execution.
    ///
    /// # Errors
    /// - [`ContractError::InvalidAmount`]: If amount is not greater than 0.
    /// - [`ContractError::FlowLimitExceeded`]: If transfer would exceed flow limits.
    /// - Any error propagated from `pay_gas_and_call_contract`.
    ///
    /// # Authorization
    /// - The `caller` must authorize.
    fn interchain_transfer(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        destination_address: Bytes,
        amount: i128,
        metadata: Option<Bytes>,
        gas_token: Option<Token>,
    ) -> Result<(), ContractError>;
}
