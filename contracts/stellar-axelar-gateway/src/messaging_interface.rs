use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};

#[contractclient(name = "AxelarGatewayMessagingClient")]
pub trait AxelarGatewayMessagingInterface {
    /// Sends a message to the specified destination chain and contarct address with a given payload.
    ///
    /// This function is the entry point for general message passing between chains.
    ///
    /// A registered chain name on Axelar must be used for `destination_chain`.
    fn call_contract(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    );

    /// Checks if a message is approved
    ///
    /// Determines whether a given message, identified by its `source_chain` and `message_id`, is approved.
    ///
    /// Returns true if a message with the given `payload_hash`  is approved.
    fn is_message_approved(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Checks if a message is executed.
    ///
    /// Returns true if the message is executed, false otherwise.
    fn is_message_executed(env: Env, source_chain: String, message_id: String) -> bool;

    /// Validates if a message is approved. If message was in approved status, status is updated to executed to avoid
    /// replay.
    ///
    /// `caller` must be the intended `destination_address` of the contract call for validation to succeed.
    fn validate_message(
        env: Env,
        caller: Address,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool;
}
