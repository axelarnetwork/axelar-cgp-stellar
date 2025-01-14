use stellar_axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, Address, Bytes, Env, String};

use crate::error::ContractError;

#[contractclient(name = "AxelarGasServiceClient")]
pub trait AxelarGasServiceInterface {
    /// Pay for gas using a token for sending a message on a destination chain.
    ///
    /// This function is called on the source chain before calling the gateway to send a message.
    ///
    /// `sender` refers to the address that sent the cross-chain message via the `axelar_gateway`.
    /// The `spender` pays the gas but might differ from the `sender`,
    /// e.g. the `sender` is a contract, but the `spender` can be the user signing the transaction.
    fn pay_gas(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        spender: Address,
        token: Token,
        metadata: Bytes,
    ) -> Result<(), ContractError>;

    /// Adds additional gas payment after initiating a cross-chain message.
    ///
    /// `sender` refers to the address that sent the cross-chain message via the `axelar_gateway`.
    /// The `spender` pays the gas but might differ from the `sender`,
    /// e.g. the `sender` is a contract, but the `spender` can be the user signing the transaction.
    fn add_gas(
        env: Env,
        sender: Address,
        message_id: String,
        spender: Address,
        token: Token,
    ) -> Result<(), ContractError>;

    /// Allows the `gas_collector` to collect accumulated fees from the contract.
    ///
    /// Only callable by the `gas_collector`.
    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError>;

    /// Refunds gas payment to the receiver in relation to a specific cross-chain message.
    ///
    /// Only callable by the `gas_collector`.
    fn refund(env: Env, message_id: String, receiver: Address, token: Token);

    /// Returns the address of the `gas_collector`.
    fn gas_collector(env: &Env) -> Address;
}
