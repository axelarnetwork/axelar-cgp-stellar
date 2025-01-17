//! InterchainTokenExecutable
//!
//! This is an executable interface that accepts an interchain token from ITS contract
//! along with an arbitrary message.
//!
//! This is similar to the [AxelarExecutableInterface] but meant for messages sent with an ITS token.

use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};

pub trait CustomExecutableInterface {
    type Error: Into<soroban_sdk::Error>;
    fn interchain_token_service(env: &Env) -> Address;
    fn execute(
        env: &Env,
        source_chain: String,
        message_id: String,
        source_address: Bytes,
        payload: Bytes,
        token_id: BytesN<32>,
        token_address: Address,
        amount: i128,
    ) -> Result<(), Self::Error>;
}

/// Marker trait for interfaces that should not be implemented manually.
#[doc(hidden)]
pub trait DeriveOnly {}

/// Interface for an Interchain Token Executable app. Use the [axelar_soroban_std::Executable] derive macro to implement this interface.
#[contractclient(name = "InterchainTokenExecutableClient")]
pub trait InterchainTokenExecutableInterface: CustomExecutableInterface + DeriveOnly {
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
    ) -> Result<(), soroban_sdk::Error>;
}
