//! InterchainTokenExecutable
//!
//! This is an executable interface that accepts an interchain token from ITS contract
//! along with an arbitrary message.
//!
//! This is similar to the [`AxelarExecutableInterface`](stellar_axelar_gateway::executable::AxelarExecutableInterface) but meant for messages sent with an ITS token.

use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};
use stellar_axelar_std::derive_only;
pub use stellar_axelar_std::InterchainTokenExecutable;

/// This trait must be implemented by a contract to be compatible with the [`InterchainTokenExecutableInterface`].
///
/// To make a contract executable by the interchain token service contract, it must implement the [`InterchainTokenExecutableInterface`] trait.
/// For security purposes and convenience, sender authorization and other commonly shared code necessary to implement that trait can be automatically generated with the [`InterchainTokenExecutable`] derive macro.
/// All parts that are specific to an individual ITS executable contract are collected in this [`CustomInterchainTokenExecutable`] trait and must be implemented by the contract to be compatible with the [`InterchainTokenExecutableInterface`] trait.
///
/// Do NOT add the implementation of [`CustomInterchainTokenExecutable`] to the public interface of the contract, i.e. do not annotate the `impl` block with `#[contractimpl]`
pub trait CustomInterchainTokenExecutable {
    /// The type of error the [`CustomInterchainTokenExecutable::__authorized_execute_with_token`] function returns. Generally matches the error type of the whole contract.
    type Error: Into<soroban_sdk::Error>;

    /// Returns the address of the interchain token service contract that is authorized to execute arbitrary payloads on this contract
    fn __interchain_token_service(env: &Env) -> Address;

    /// The custom execution logic that takes in an arbitrary payload and a token.
    /// At the time this function is called, the calling address has already been verified as the correct interchain token service contract.
    fn __authorized_execute_with_token(
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

derive_only!();

/// Interface for an Interchain Token Executable app. Use the [`InterchainTokenExecutable`] derive macro to implement this interface.
///
/// **DO NOT IMPLEMENT THIS MANUALLY!**
#[contractclient(name = "InterchainTokenExecutableClient")]
pub trait InterchainTokenExecutableInterface: CustomInterchainTokenExecutable + DeriveOnly {
    /// Returns the address of the interchain token service contract that is authorized to execute arbitrary payloads on this contract
    fn interchain_token_service(env: &Env) -> Address;

    /// Execute a cross-chain message with the given payload and token.
    /// # Authorization
    /// - Only callable by ITS contract.
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
