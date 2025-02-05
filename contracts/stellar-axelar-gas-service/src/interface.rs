use soroban_sdk::{contractclient, Address, Bytes, Env, String};
use stellar_axelar_std::interfaces::OperatableInterface;
use stellar_axelar_std::types::Token;

use crate::error::ContractError;

#[contractclient(name = "AxelarGasServiceClient")]
pub trait AxelarGasServiceInterface: OperatableInterface {
    /// Pay for gas using a token for sending a message on a destination chain.
    ///
    /// This function is called on the source chain before calling the gateway to send a message.
    /// The `spender` pays the gas but might differ from the `sender`,
    /// e.g. the `sender` is a contract, but the `spender` can be the user signing the transaction.
    ///
    /// # Arguments
    /// * `sender` - The address initiating the gas payment. It's the address that sent the cross-chain message via the `axelar_gateway`.
    /// * `destination_chain` - The destination chain for the message.
    /// * `destination_address` - The destination contract address for the message.
    /// * `payload` - The payload data associated with the message.
    /// * `spender` - The address of the spender paying for the gas. Might differ from the `sender`. Excess gas will be refunded to this address.
    /// * `token` - The token used to pay for the gas, including the address and amount.
    /// * `metadata` - Additional metadata associated with the gas payment.
    ///
    /// # Errors
    /// - [`ContractError::InvalidAmount`]: If the token amount is zero or negative.
    ///
    /// # Authorization
    /// - The `spender` address must authorize the token transfer to the gas service.
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
    /// * `sender` - The address that sent the cross-chain message.
    /// * `message_id` - The identifier of the message for which gas is being added.
    /// * `spender` - The address of the spender paying for the gas.
    /// * `token` - The token used to pay for the gas, including the address and amount.
    ///
    /// # Errors
    /// - [`ContractError::InvalidAmount`]: If the token amount is zero or negative.
    ///
    /// # Authorization
    /// - The `spender` address must authorize.
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
    /// * `receiver` - The address that will receive the collected feeds.
    /// * `token` - The token address and amount for the fee collection.
    ///
    /// # Errors
    /// - [`ContractError::InvalidAmount`]: If the token amount is zero or negative.
    /// - [`ContractError::InsufficientBalance`]: If the contract's token balance is insufficient to cover the transfer.
    ///
    /// # Authorization
    /// - [`OperatableInterface::operator`] must authorize.
    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError>;

    /// Refunds gas payment to the specified receiver in relation to a specific cross-chain message.
    ///
    /// # Arguments
    /// * `message_id` - The identifier of the cross-chain message for which the gas fees are being refunded.
    /// * `receiver` - The address of the receiver to whom the gas fees will be refunded.
    /// * `token` - The token used for the refund, including the address and amount.
    ///
    /// # Authorization
    /// - [`OperatableInterface::operator`] must authorize.
    fn refund(env: Env, message_id: String, receiver: Address, token: Token);
}
