//! InterchainTokenExecutable
//!
//! This is an executable interface that accepts an interchain token from ITS contract
//! along with an arbitrary message.
//!
//! This is similar to the [AxelarExecutableInterface] but meant for messages sent with an ITS token.

use soroban_sdk::{Address, Bytes, BytesN, Env, String};

/// Interface for an Interchain Token Executable app.
pub trait InterchainTokenExecutableInterface {
    type Error: Into<soroban_sdk::Error>;

    /// Return the trusted interchain token service contract address.
    fn interchain_token_service(env: &Env) -> Address;

    /// Execute a cross-chain message with the given payload and token.
    /// [`validate`] must be called first in the implementation of [`execute_with_interchain_token`].
    fn execute_with_interchain_token(
        env: &Env,
        source_chain: String,
        message_id: String,
        source_address: Bytes,
        payload: Bytes,
        token_id: BytesN<32>,
        token_address: Address,
        amount: i128,
    ) -> Result<(), <Self as InterchainTokenExecutableInterface>::Error>;

    /// Ensure that only the interchain token service can call [`execute_with_interchain_token`].
    fn validate(env: &Env) -> Result<(), <Self as InterchainTokenExecutableInterface>::Error> {
        Self::interchain_token_service(env).require_auth();
        Ok(())
    }
}
