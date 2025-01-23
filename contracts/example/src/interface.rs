use soroban_sdk::{Address, Bytes, BytesN, Env, String};
use crate::contract::ExampleError;
use stellar_axelar_std::types::Token;

pub trait ExampleInterface {
    /// Retrieves the address of the gas service.
    fn gas_service(env: &Env) -> Address;

    /// Sends a message to a specified destination chain.
    ///
    /// The function also handles the payment of gas for the cross-chain transaction.
    ///
    /// # Arguments
    /// * `caller` - The address of the caller initiating the message.
    /// * `destination_chain` - The name of the destination chain where the message will be sent.
    /// * `destination_address` - The address on the destination chain where the message will be sent.
    /// * `message` - The message to be sent.
    /// * `gas_token` - The token used to pay for gas during the transaction.
    ///
    /// # Returns
    /// - `Ok(())`
    ///
    /// # Errors
    /// - Any error propagated from the `pay_gas` or `call_contract` functions.
    ///
    /// # Authorization
    /// - The `caller` address must authenticate.
    fn send(
        env: &Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        message: Bytes,
        gas_token: Token,
    );

    /// Sends a token to a specified destination chain.
    ///
    /// The function also emits an event upon successful transfer.
    ///
    /// # Arguments
    /// * `caller` - The address of the caller initiating the token transfer.
    /// * `token_id` - The ID of the token to be transferred.
    /// * `destination_chain` - The name of the destination chain where the token will be sent.
    /// * `destination_app_contract` - The address of the application contract on the destination chain.
    /// * `amount` - The amount of the token to be transferred.
    /// * `recipient` - An optional recipient address on the destination chain.
    /// * `gas_token` - The token used to pay for gas during the transaction.
    ///
    /// # Returns
    /// - `Ok(())`
    ///
    /// # Errors
    /// - Any errors propagated during the `interchain_transfer` function.
    ///
    /// # Authorization
    /// - The `caller` address must authenticate.
    fn send_token(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        destination_app_contract: Bytes,
        amount: i128,
        recipient: Option<Bytes>,
        gas_token: Token,
    ) -> Result<(), ExampleError>;
}