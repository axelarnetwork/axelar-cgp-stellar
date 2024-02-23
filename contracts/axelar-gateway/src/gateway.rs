use soroban_sdk::{bytes, bytesn, contract, contracterror, contractimpl, contracttype, log, panic_with_error, xdr::{FromXdr, ToXdr}, Address, Bytes, BytesN, Env, String, Symbol, Vec
};
use crate::admin::*;


#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Data {
    pub chain_id: u64,
    pub commandids: Vec<BytesN<32>>,
    pub commands: Vec<Bytes>,
    pub params: Vec<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Input {
    pub data: Data,
    pub proof: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractPayload {
    pub src_chain: String,
    pub src_addr: String,
    pub contract: String,
    // contract address
    pub payload_ha: BytesN<32>,
    // payload hash
    pub src_tx_ha: BytesN<32>,
    // source tx hash
    pub src_evnt: u64, // source event index // do u256 instead?
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCallApprovedEvent {
    pub src_chain: String,
    pub src_addr: String,
    pub src_tx: BytesN<32>,
    // source tx hash
    pub src_event: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCallApprovedKey {
    pub prefix: Symbol,
    pub command_id: BytesN<32>,
    pub src_chain: String,
    pub src_addr: String,
    pub contract: String,
    // contract address
    pub payload_ha: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCall {
    pub prefix: Symbol,
    pub dest_chain: Bytes,
    pub dest_addr: Bytes,
    pub payload: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandExecuted {
    pub prefix: Symbol,
    pub command_id: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutedEvent {
    pub command_id: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Operatorship {
    pub new_ops: Vec<BytesN<65>>,
    // new_operators
    pub new_wghts: Vec<u128>,
    // new_weights
    pub new_thres: u128, // new_threshold
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Validate {
    pub operators: Vec<BytesN<65>>,
    pub weights: Vec<u128>,
    // uint256
    pub threshold: u128,
    // uint256
    pub signatures: Vec<BytesN<65>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedMsg {
    pub text: Symbol,
    pub hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefixHash {
    pub prefix: Symbol,
    pub hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefixEpoch {
    pub prefix: Symbol,
    pub epoch: u128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidOperators = 1,
    InvalidWeights = 2,
    InvalidThreshold = 3,
    DuplicateOperators = 4,
    LowSignaturesWeight = 5,
    InvalidCommands = 6,
    InvalidOrdering = 7,
}

#[contract]
pub struct Gateway;

#[contractimpl]
impl Gateway {
    pub fn initialize(env: Env, admin: Address, recent_ops: Bytes) {
        if has_administrator(&env) {
            panic!("already initialized")
        }
        write_administrator(&env, &admin);

        Self::transfer_op(env.clone(), recent_ops);
    }

    pub fn execute(
        env: Env,
        input: Bytes,
    ) {

        // transferOperatorship converted into Bytes, and then keccak256 hashed.
        let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().keccak256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));
        // approveContractCall converted into Bytes, and then keccak256 hashed.
        let SELECTOR_APPROVE_CONTRACT_CALL: BytesN<32> = env.crypto().keccak256(&bytes!(&env, 0x617070726f7665436f6e747261637443616c6c));

        let decoded: Input = Input::from_xdr(&env, &input).unwrap();
        let data: Data = decoded.data;
        let proof: Bytes = decoded.proof;
        let hash: BytesN<32> = env.crypto().keccak256(&data.clone().to_xdr(&env));
        let signed_message_hash: BytesN<32> = Self::to_signed_msg_hsh(env.clone(), hash);
        let mut allow_operatorship_transfership: bool = Self::validate_proof(env.clone(), signed_message_hash, proof.clone());

        let _chain_id: u64 = data.chain_id;
        let command_ids: Vec<BytesN<32>> = data.commandids;
        let commands: Vec<Bytes> = data.commands;
        let params: Vec<Bytes> = data.params;

        let commands_length: u32 = command_ids.len();


        if commands_length != commands.len() || commands_length != params.len() {
            panic_with_error!(env, Error::InvalidCommands);
        }

        for i in 0..commands_length {
            let command_id: BytesN<32> = command_ids.get(i).unwrap();
            let command_hash: BytesN<32> = env.crypto().keccak256(&commands.get(i).unwrap());
            let mut success: bool = false;

            if command_hash == SELECTOR_TRANSFER_OPERATORSHIP {
                if !allow_operatorship_transfership {
                    continue;
                }
                allow_operatorship_transfership = false;
                Self::_setCommandExecuted(env.clone(), command_id.clone(), true);
                success = Self::transfer_op(env.clone(), params.get(i).unwrap());
            } else if command_hash == SELECTOR_APPROVE_CONTRACT_CALL {
                Self::_setCommandExecuted(env.clone(), command_id.clone(), true);
                success = Self::approve_cc(env.clone(), params.get(i).unwrap(), command_id.clone());
            }

            if success {
                let event: ExecutedEvent = ExecutedEvent { command_id: command_id.clone() };
                env.events().publish((), event);
            } else {
                Self::_setCommandExecuted(env.clone(), command_id.clone(), false);
            }
        }
    }

    fn approve_cc(
        env: Env,
        params: Bytes,
        command_id: BytesN<32>,
    ) -> bool {
        let decoded: ContractPayload = ContractPayload::from_xdr(&env, &params).unwrap();
        let src_chain: String = decoded.src_chain;
        let src_addr: String = decoded.src_addr;
        let contract: String = decoded.contract;
        let payload_ha: BytesN<32> = decoded.payload_ha;
        let src_tx: BytesN<32> = decoded.src_tx_ha;
        let src_event: u64 = decoded.src_evnt;

        Self::_setContractCallApproved(env.clone(), command_id.clone(), src_chain.clone(), src_addr.clone(), contract.clone(), payload_ha.clone());
        let event: ContractCallApprovedEvent = ContractCallApprovedEvent { src_chain, src_addr, src_tx, src_event };
        env.events().publish((command_id, contract, payload_ha), event);

        true
    }

    fn _setContractCallApproved(
        env: Env,
        commandId: BytesN<32>,
        sourceChain: String,
        sourceAddress: String,
        contractAddress: String,
        payloadHash: BytesN<32>,
    ) {
        let key: BytesN<32> = Self::_getIsContractCallApprovedKey(env.clone(), commandId.clone(), sourceChain.clone(), sourceAddress.clone(), contractAddress.clone(), payloadHash.clone());
        env.storage().instance().set(&key, &true);
    }

    fn _getIsContractCallApprovedKey(
        env: Env,
        commandId: BytesN<32>,
        sourceChain: String,
        sourceAddress: String,
        contractAddress: String,
        payloadHash: BytesN<32>,
    ) -> BytesN<32> {
        let data: ContractCallApprovedKey = ContractCallApprovedKey {
            prefix: Symbol::new(&env, &"approved"),
            command_id: commandId,
            src_chain: sourceChain,
            src_addr: sourceAddress,
            contract: contractAddress,
            payload_ha: payloadHash,
        };
        let key: BytesN<32> = env.crypto().keccak256(&data.to_xdr(&env));

        key
    }

    fn _setCommandExecuted(
        env: Env,
        command_id: BytesN<32>,
        executed: bool,
    ) {
        let data: CommandExecuted = CommandExecuted {
            prefix: Symbol::new(&env, &"executed"),
            command_id: command_id,
        };
        let key: BytesN<32> = env.crypto().keccak256(&data.to_xdr(&env));
        env.storage().instance().set(&key, &executed);
    }

    pub fn call_contract(
        env: Env,
        caller: Address,
        dest_chain: Bytes,
        dest_addr: Bytes,
        payload: Bytes,
    ) {
        caller.require_auth();

        let data: ContractCall = ContractCall {
            prefix: Symbol::new(&env, "ContractCall"),
            dest_chain,
            dest_addr,
            payload: payload.clone(),
        };
        env.events().publish((caller, env.crypto().keccak256(&payload)), data);
    }

    fn transfer_op( // transferOperatorship
                    env: Env,
                    params: Bytes,
    ) -> bool {
        log!(env, "transfer_op 1");
        let tokens: Operatorship = Operatorship::from_xdr(&env, &params).unwrap();
        let new_operators: Vec<BytesN<65>> = tokens.new_ops;
        let new_weights: Vec<u128> = tokens.new_wghts;
        let new_threshold: u128 = tokens.new_thres;

        let operators_length: u32 = new_operators.len();
        let weights_length: u32 = new_weights.len();

        if operators_length == 0 || !Self::is_sorted_asc_no_dup(env.clone(), new_operators.clone())
        {
            panic_with_error!(env, Error::InvalidOperators);
        }

        if weights_length != operators_length {
            panic_with_error!(env, Error::InvalidWeights);
        }

        let mut total_weight: u128 = 0;

        for i in 0..weights_length {
            total_weight += new_weights.get(i).unwrap();
        }

        if new_threshold == 0 || total_weight < new_threshold {
            panic_with_error!(env, Error::InvalidThreshold);
        }

        let new_operators_hash: BytesN<32> = env.crypto().keccak256(&params);

        let new_operators_hash_key: BytesN<32> = env.crypto().keccak256(&PrefixHash { prefix: Symbol::new(&env, &"operators_for_epoch"), hash: new_operators_hash.clone() }.to_xdr(&env));
        let existing_epoch: u128 = env.storage().instance().get(&new_operators_hash_key).unwrap_or(0);

        if existing_epoch > 0 {
            panic_with_error!(env, Error::DuplicateOperators);
        }

        let epoch: u128 = env.storage().instance().get(&Symbol::new(&env, &"current_epoch")).unwrap_or(0) + 1;
        env.storage().instance().set(&Symbol::new(&env, &"current_epoch"), &epoch);
        env.storage().instance().set(&PrefixEpoch { prefix: Symbol::new(&env, &"epoch_for_operators"), epoch }, &new_operators_hash);
        env.storage().instance().set(&new_operators_hash_key, &epoch);

        let event: Operatorship = Operatorship { new_ops: new_operators, new_wghts: new_weights, new_thres: new_threshold };
        env.events().publish((), event);

        return true;
    }

    pub fn validate_contract_call(
        env: Env,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        contract_address: String,
        payload_hash: BytesN<32>,
    ) -> bool {
        // do require_auth here?
        let key: BytesN<32> = Self::_getIsContractCallApprovedKey(env.clone(), command_id.clone(), source_chain.clone(), source_address.clone(), contract_address.clone(), payload_hash.clone());
        let valid: bool = env.storage().instance().get(&key).unwrap_or(false);

        if valid {
            env.storage().instance().set(&key, &false);
        }

        valid
    }

    pub fn to_signed_msg_hsh(
        env: Env,
        hash: BytesN<32>,
    ) -> BytesN<32> {
        let data: SignedMsg = SignedMsg {
            text: Symbol::new(&env, &"Soroban"),
            hash: hash,
        };
        return env.crypto().keccak256(&data.to_xdr(&env));
    }

    pub fn validate_proof(
        env: Env,
        msghash: BytesN<32>,
        proof: Bytes,
    ) -> bool {
        const OLD_KEY_RETENTION: u128 = 16;

        let tokens: Validate = Validate::from_xdr(&env, &proof).unwrap();
        let operators: Vec<BytesN<65>> = tokens.operators;
        let weights: Vec<u128> = tokens.weights;
        let threshold: u128 = tokens.threshold;
        let signatures: Vec<BytesN<65>> = tokens.signatures;

        // Three parts of operators is treated as constant
        let operators_data: Operatorship = Operatorship {
            new_ops: operators.clone(),
            new_wghts: weights.clone(),
            new_thres: threshold,
        };
        let operators_hash: BytesN<32> = env.crypto().keccak256(&operators_data.to_xdr(&env));
        let operators_hash_key: BytesN<32> = env.crypto().keccak256(&PrefixHash { prefix: Symbol::new(&env, &"operators_for_epoch"), hash: operators_hash.clone() }.to_xdr(&env));

        let operators_epoch: u128 = env.storage().instance().get(&operators_hash_key).unwrap_or(0); //uint256
        let epoch: u128 = env.storage().instance().get(&Symbol::new(&env, &"current_epoch")).unwrap_or(0); //uint256

        if operators_epoch == 0 || epoch - operators_epoch >= OLD_KEY_RETENTION {
            panic_with_error!(env, Error::InvalidOperators);
        }

        Self::validate_sig(env, msghash, operators, weights, threshold, signatures);

        return operators_epoch == epoch;
    }

    fn validate_sig(
        env: Env,
        msghash: BytesN<32>,
        public_keys: Vec<BytesN<65>>, // operators
        weights: Vec<u128>,
        threshold: u128,
        signatures: Vec<BytesN<65>>,
    ) {
        let mut weight: u128 = 0;
        let signatures_len: u32 = signatures.len();


        let mut prev_index = 0;
        for i in 0..signatures_len {
            let signature = &signatures.get(i).unwrap();
            let psig: [u8; 64] = signature.to_array()[..64].try_into().unwrap();
            let main_sig: BytesN<64> = BytesN::from_array(&env, &psig);
            let public_key = env.crypto().secp256k1_recover(
                &msghash,
                &main_sig,
                signature.last().unwrap() as u32 - 27,
            );

            let public_key_idx = public_keys.first_index_of(&public_key).unwrap_or(0);

            // check that signature's public key index is greater than the previous index, aside from first iteration
            if i > 0 && !(public_key_idx > prev_index) {
                panic_with_error!(env, Error::InvalidOrdering);
            }
            prev_index = public_key_idx;

            // return if weight sum above threshold
            weight += weights.get(public_key_idx).unwrap();
            // weight needs to reach or surpass threshold
            if weight >= threshold {
                return;
            }
        }
        // if weight sum below threshold
        panic_with_error!(env, Error::LowSignaturesWeight);
    }

    fn is_sorted_asc_no_dup(
        env: Env,
        accounts: Vec<BytesN<65>>,
    ) -> bool {
        for i in 0..accounts.len() - 1 {
            if accounts.get(i).unwrap() >= accounts.get(i + 1).unwrap() {
                return false;
            }
        }

        return accounts.get(0).unwrap() != bytesn!(&env, 0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000);
    }
}