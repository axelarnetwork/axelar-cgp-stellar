use soroban_sdk::{contractclient, Address, Bytes, Env, String};
use stellar_axelar_std::types::Token;

use crate::error::ContractError;

#[contractclient(name = "AxelarGasServiceClient")]
pub trait AxelarGasServiceInterface {
    /// Pay for gas using a token for sending a message on a destination chain.
    ///
    /// This function is called on the source chain before calling the gateway to send a message.
    /// The `spender` pays the gas but might differ from the `sender`,
    /// e.g. the `sender` is a contract, but the `spender` can be the user signing the transaction.
    ///
    /// # Arguments
    /// * `sender` - The address initiating the gas payment. It's the address that sent the cross-chain message via the `axelar_gateway`.
    /// * `destination_chain` - The name of the destination chain where the transaction will be executed.
    /// * `destination_address` - The address on the destination chain where the transaction will be executed.
    /// * `payload` - The payload data associated with the transaction.
    /// * `spender` - The address of the spender paying for the gas. Might differ from the `sender`.
    /// * `token` - The token used to pay for the gas, including the address and amount.
    /// * `metadata` - Additional metadata associated with the gas payment.
    ///
    /// # Returns
    /// - `Ok(())`
    ///
    /// # Errors
    /// - `ContractError::InvalidAmount`: If the token amount is zero or negative.
    /// - Any error propagated from the token transfer operation.
    ///
    /// # Authorization
    /// - The `spender` address must authenticate.
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
    /// The `spender` pays the gas but might differ from the `sender`,
    /// e.g. the `sender` is a contract, but the `spender` can be the user signing the transaction.
    ///
    /// # Arguments
    /// * `sender` - The address of the sender initiating the gas addition. It's the address that sent the cross-chain message via the `axelar_gateway`.
    /// * `message_id` - The identifier of the message for which gas is being added.
    /// * `spender` - The address of the spender paying for the gas.
    /// * `token` - The token used to pay for the gas, including the address and amount.
    ///
    /// # Returns
    /// - `Ok(())`
    ///
    /// # Errors
    /// - `ContractError::InvalidAmount`: If the token amount is zero or negative.
    /// - Any error propagated from the token transfer operation.
    ///
    /// # Authorization
    /// - The `spender` address must authenticate.
    fn add_gas(
        env: Env,
        sender: Address,
        message_id: String,
        spender: Address,
        token: Token,
    ) -> Result<(), ContractError>;

    /// Collects gas fees and transfers them to a specified receiver.
    ///
    /// Allows the `gas_collector` to collect accumulated fees from the contract.
    ///
    /// # Arguments
    /// * `receiver` - The address of the receiver to whom the collected fees will be transferred.
    /// * `token` - The token used for the fee collection, including the address and amount.
    ///
    /// # Returns
    /// - `Ok(())`
    ///
    /// # Errors
    /// - `ContractError::InvalidAmount`: If the token amount is zero or negative.
    /// - `ContractError::InsufficientBalance`: If the contract's token balance is insufficient to cover the transfer.
    /// - Any error propagated from the token transfer operation.
    ///
    /// # Authorization
    /// - The gas collector must authenticate.
    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError>;

    /// Refunds gas payment to the specified receiver in relation to a specific cross-chain message.
    ///
    /// # Arguments
    /// * `message_id` - The identifier of the cross-chain message for which the gas fees are being refunded.
    /// * `receiver` - The address of the receiver to whom the gas fees will be refunded.
    /// * `token` - The token used for the refund, including the address and amount.
    ///
    /// # Returns
    /// - `Ok(())`: Returns `Ok` if the refund is successful.
    ///
    /// # Errors
    /// - Any error propagated from the token transfer operation.
    ///
    /// # Authorization
    /// - The `gas_collector` must authenticate.
    fn refund(env: Env, message_id: String, receiver: Address, token: Token);

    /// Returns the address of the `gas_collector`.
    fn gas_collector(env: &Env) -> Address;
}
