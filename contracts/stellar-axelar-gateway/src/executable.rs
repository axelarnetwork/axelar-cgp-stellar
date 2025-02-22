use soroban_sdk::{Address, Bytes, Env, String};
use stellar_axelar_std::{derive_only, ensure};

use crate::AxelarGatewayMessagingClient;

/// Interface for an Axelar Executable app.
pub trait AxelarExecutableInterface: CustomAxelarExecutableInterface + DeriveOnly {
    /// Return the trusted gateway contract id.
    fn gateway(env: &Env) -> Address;

    /// Execute a cross-chain message with the given payload. This function must validate that the message is received from the trusted gateway.
    fn execute(
        env: &Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) -> Result<(), <Self as CustomAxelarExecutableInterface>::Error>;
}

derive_only!();

pub trait CustomAxelarExecutableInterface {
    type Error: Into<soroban_sdk::Error>;

    fn __gateway(env: &Env) -> Address;
    fn __validated_execute(
        env: &Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) -> Result<(), Self::Error>;
}

/// Validate if a gateway has approved a message.
/// This should be called from an implementation of `execute` before executing custom app logic.
/// This method doesn't get exposed from the contract, as Soroban SDK's contractimpl macro ignores default trait methods.
pub fn validate_message<T: CustomAxelarExecutableInterface>(
    env: &Env,
    source_chain: &String,
    message_id: &String,
    source_address: &String,
    payload: &Bytes,
) -> Result<(), ValidationError> {
    let gateway = AxelarGatewayMessagingClient::new(env, &T::__gateway(env));

    // Validate that the message was approved by the gateway
    ensure!(
        gateway.validate_message(
            &env.current_contract_address(),
            source_chain,
            message_id,
            source_address,
            &env.crypto().keccak256(payload).into(),
        ),
        ValidationError::NotApproved
    );

    Ok(())
}

pub enum ValidationError {
    NotApproved,
}
