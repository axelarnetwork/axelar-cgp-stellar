use axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, Address, Bytes, Env, String};

use crate::error::ContractError;

#[contractclient(name = "AxelarGasServiceClient")]
pub trait AxelarGasServiceInterface {
    /// Pay for gas using a token for a contract call on a destination chain.
    ///
    /// This function is called on the source chain before calling the gateway to execute a remote contract.
    fn pay_gas_for_contract_call(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        refund_address: Address,
        token: Token,
    ) -> Result<(), ContractError>;

    /// Add additional gas payment after initiating a cross-chain call.
    fn add_gas(
        env: Env,
        sender: Address,
        message_id: String,
        token: Token,
        refund_address: Address,
    ) -> Result<(), ContractError>;

    /// Allows the `gas_collector` to collect accumulated fees from the contract.
    ///
    /// Only callable by the `gas_collector`.
    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError>;

    /// Refunds gas payment to the receiver in relation to a specific cross-chain transaction.
    ///
    /// Only callable by the `gas_collector`.
    fn refund(env: Env, message_id: String, receiver: Address, token: Token);

    /// Returns the address of the `gas_collector`.
    fn gas_collector(env: &Env) -> Address;
}
