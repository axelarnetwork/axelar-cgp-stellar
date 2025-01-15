use soroban_sdk::{contractclient, Address, Bytes, Env, String};
use stellar_axelar_std::ensure;

use crate::AxelarGatewayMessagingClient;
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ExecutableError {
    NotApproved = 1,
}

/// Interface for an Axelar Executable app.
#[contractclient(name = "AxelarExecutableClient")]
pub trait AxelarExecutableInterface {
    /// Return the trusted gateway contract id.
    fn gateway(env: &Env) -> Address;

    /// Execute a cross-chain message with the given payload. This function must validate that the message is received from the trusted gateway.
    fn execute(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    );

    /// Validate if a gateway has approved a message.
    /// This should be called from an implementation of `execute` before executing custom app logic.
    /// This method doesn't get exposed from the contract, as Soroban SDK's contractimpl macro ignores default trait methods.
    fn validate_message(
        env: &Env,
        source_chain: &String,
        message_id: &String,
        source_address: &String,
        payload: &Bytes,
    ) -> Result<(), ExecutableError> {
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
            ExecutableError::NotApproved
        );

        Ok(())
    }
}
