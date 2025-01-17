use soroban_sdk::{Address, Bytes, Env, String};
use stellar_axelar_std::ensure;

use crate::AxelarGatewayMessagingClient;

pub trait NotApprovedError {
    fn not_approved() -> Self;
}

/// Generates the implementation for the [`NotApprovedError`] trait for the given error type
#[macro_export]
macro_rules! impl_not_approved_error {
    ($error:ident) => {
        impl NotApprovedError for $error {
            fn not_approved() -> Self {
                Self::NotApproved
            }
        }
    };
}

/// Interface for an Axelar Executable app.
pub trait AxelarExecutableInterface {
    type Error: Into<soroban_sdk::Error> + NotApprovedError;

    /// Return the trusted gateway contract id.
    fn gateway(env: &Env) -> Address;

    /// Execute a cross-chain message with the given payload. This function must validate that the message is received from the trusted gateway.
    fn execute(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) -> Result<(), <Self as AxelarExecutableInterface>::Error>;

    /// Validate if a gateway has approved a message.
    /// This should be called from an implementation of `execute` before executing custom app logic.
    /// This method doesn't get exposed from the contract, as Soroban SDK's contractimpl macro ignores default trait methods.
    fn validate_message(
        env: &Env,
        source_chain: &String,
        message_id: &String,
        source_address: &String,
        payload: &Bytes,
    ) -> Result<(), <Self as AxelarExecutableInterface>::Error> {
        let gateway = AxelarGatewayMessagingClient::new(env, &Self::gateway(env));

        // Validate that the message was approved by the gateway
        ensure!(
            gateway.validate_message(
                &env.current_contract_address(),
                source_chain,
                message_id,
                source_address,
                &env.crypto().keccak256(payload).into(),
            ),
            Self::Error::not_approved()
        );

        Ok(())
    }
}
