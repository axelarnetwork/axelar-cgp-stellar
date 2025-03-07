use soroban_sdk::{Address, Bytes, Env, String};
pub use stellar_axelar_std::AxelarExecutable;
use stellar_axelar_std::{derive_only, ensure};

use crate::AxelarGatewayMessagingClient;

derive_only!();

/// Interface for an Axelar Executable app. Use the [`AxelarExecutable`] derive macro to implement this interface.
///
/// **DO NOT IMPLEMENT THIS MANUALLY!**
pub trait AxelarExecutableInterface: CustomAxelarExecutable + DeriveOnly {
    /// Return the trusted gateway contract id.
    fn gateway(env: &Env) -> Address;

    /// Execute a cross-chain message with the given payload. This function must validate that the message is received from the trusted gateway.
    fn execute(
        env: &Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) -> Result<(), <Self as CustomAxelarExecutable>::Error>;
}

/// Encapsulates the logic for executing a cross-chain message. This trait must be implemented by a contract to be compatible with the [`AxelarExecutableInterface`].
///
/// Do NOT add the implementation of [`CustomAxelarExecutable`] to the public interface of the contract, i.e. do not annotate the `impl` block with `#[contractimpl]`
pub trait CustomAxelarExecutable {
    type Error: Into<soroban_sdk::Error>;

    /// Custom implementation of the gateway query function that's called by [`AxelarExecutableInterface::gateway`].
    fn __gateway(env: &Env) -> Address;

    /// Custom implementation of the execute function that's called by [`AxelarExecutableInterface::execute`] after validation has succeeded.
    /// It is guaranteed that the [`validate_message`] function has already been called when this function is executed.
    fn __execute(
        env: &Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) -> Result<(), Self::Error>;
}

/// Validate if a gateway has approved a message.
/// This is called as part of the generated implementation of [`AxelarExecutableInterface::execute`] before running [`CustomAxelarExecutable::__execute`].
pub fn validate_message<T: CustomAxelarExecutable>(
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
