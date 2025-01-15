use crate::auth;
use crate::error::ContractError;
use crate::event::{ContractCalledEvent, MessageApprovedEvent, MessageExecutedEvent};
use crate::interface::AxelarGatewayInterface;
use crate::messaging_interface::AxelarGatewayMessagingInterface;
use crate::storage_types::{DataKey, MessageApprovalKey, MessageApprovalValue};
use crate::types::{CommandType, Message, Proof, WeightedSigners};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec};
use stellar_axelar_std::events::Event;
use stellar_axelar_std::ttl::extend_instance_ttl;
use stellar_axelar_std::{ensure, interfaces, Operatable, Ownable, Upgradable};

#[contract]
#[derive(Ownable, Upgradable, Operatable)]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGateway {
    /// Initialize the gateway
    pub fn __constructor(
        env: Env,
        owner: Address,
        operator: Address,
        domain_separator: BytesN<32>,
        minimum_rotation_delay: u64,
        previous_signers_retention: u64,
        initial_signers: Vec<WeightedSigners>,
    ) -> Result<(), ContractError> {
        interfaces::set_owner(&env, &owner);
        interfaces::set_operator(&env, &operator);

        auth::initialize_auth(
            env,
            domain_separator,
            minimum_rotation_delay,
            previous_signers_retention,
            initial_signers,
        )
    }
}

#[contractimpl]
impl AxelarGatewayMessagingInterface for AxelarGateway {
    fn call_contract(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    ) {
        caller.require_auth();

        let payload_hash = env.crypto().keccak256(&payload).into();

        ContractCalledEvent {
            caller,
            destination_chain,
            destination_address,
            payload,
            payload_hash,
        }
        .emit(&env);
    }

    fn is_message_approved(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool {
        let message_approval =
            Self::message_approval(&env, source_chain.clone(), message_id.clone());

        message_approval
            == Self::message_approval_hash(
                &env,
                Message {
                    source_chain,
                    message_id,
                    source_address,
                    contract_address,
                    payload_hash,
                },
            )
    }

    fn is_message_executed(env: Env, source_chain: String, message_id: String) -> bool {
        let message_approval = Self::message_approval(&env, source_chain, message_id);

        message_approval == MessageApprovalValue::Executed
    }

    fn validate_message(
        env: Env,
        caller: Address,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool {
        caller.require_auth();

        let key = MessageApprovalKey {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
        };
        let message_approval = Self::message_approval_by_key(&env, key.clone());
        let message = Message {
            source_chain,
            message_id,
            source_address,
            contract_address: caller,
            payload_hash,
        };

        if message_approval == Self::message_approval_hash(&env, message.clone()) {
            env.storage().persistent().set(
                &DataKey::MessageApproval(key),
                &MessageApprovalValue::Executed,
            );

            MessageExecutedEvent { message }.emit(&env);

            return true;
        }

        false
    }
}

#[contractimpl]
impl AxelarGatewayInterface for AxelarGateway {
    fn approve_messages(
        env: Env,
        messages: Vec<Message>,
        proof: Proof,
    ) -> Result<(), ContractError> {
        let data_hash: BytesN<32> = env
            .crypto()
            .keccak256(&(CommandType::ApproveMessages, messages.clone()).to_xdr(&env))
            .into();

        auth::validate_proof(&env, &data_hash, proof)?;

        ensure!(!messages.is_empty(), ContractError::EmptyMessages);

        for message in messages.into_iter() {
            let key = MessageApprovalKey {
                source_chain: message.source_chain.clone(),
                message_id: message.message_id.clone(),
            };

            // Prevent replay if message is already approved/executed
            let message_approval = Self::message_approval_by_key(&env, key.clone());
            if message_approval != MessageApprovalValue::NotApproved {
                continue;
            }

            env.storage().persistent().set(
                &DataKey::MessageApproval(key),
                &Self::message_approval_hash(&env, message.clone()),
            );

            MessageApprovedEvent { message }.emit(&env);
        }

        extend_instance_ttl(&env);

        Ok(())
    }

    fn rotate_signers(
        env: Env,
        signers: WeightedSigners,
        proof: Proof,
        bypass_rotation_delay: bool,
    ) -> Result<(), ContractError> {
        if bypass_rotation_delay {
            Self::operator(&env).require_auth();
        }

        let data_hash: BytesN<32> = signers.signers_rotation_hash(&env);

        let is_latest_signers = auth::validate_proof(&env, &data_hash, proof)?;
        ensure!(
            bypass_rotation_delay || is_latest_signers,
            ContractError::NotLatestSigners
        );

        auth::rotate_signers(&env, &signers, !bypass_rotation_delay)?;

        Ok(())
    }

    fn epoch(env: &Env) -> u64 {
        auth::epoch(env)
    }

    fn epoch_by_signers_hash(env: &Env, signers_hash: BytesN<32>) -> Result<u64, ContractError> {
        auth::epoch_by_signers_hash(env, signers_hash)
    }

    fn signers_hash_by_epoch(env: &Env, epoch: u64) -> Result<BytesN<32>, ContractError> {
        auth::signers_hash_by_epoch(env, epoch)
    }

    fn validate_proof(
        env: &Env,
        data_hash: BytesN<32>,
        proof: Proof,
    ) -> Result<bool, ContractError> {
        auth::validate_proof(env, &data_hash, proof)
    }
}

impl AxelarGateway {
    /// Get the message approval value by `source_chain` and `message_id`, defaulting to `MessageNotApproved`
    fn message_approval(
        env: &Env,
        source_chain: String,
        message_id: String,
    ) -> MessageApprovalValue {
        let key = MessageApprovalKey {
            source_chain,
            message_id,
        };

        Self::message_approval_by_key(env, key)
    }

    /// Get the message approval value by key, defaulting to `MessageNotApproved`
    fn message_approval_by_key(env: &Env, key: MessageApprovalKey) -> MessageApprovalValue {
        env.storage()
            .persistent()
            .get(&DataKey::MessageApproval(key))
            .unwrap_or(MessageApprovalValue::NotApproved)
    }

    fn message_approval_hash(env: &Env, message: Message) -> MessageApprovalValue {
        MessageApprovalValue::Approved(env.crypto().keccak256(&message.to_xdr(env)).into())
    }

    // Modify this function to add migration logic
    const fn run_migration(_env: &Env, _migration_data: ()) {}
}
