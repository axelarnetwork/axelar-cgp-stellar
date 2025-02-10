use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate as stellar_axelar_std;
use crate::contractstorage;

#[contract]
pub struct Contract;

mod storage {

    use super::*;

    #[contractstorage]
    enum DataKey {
        #[instance]
        #[value(u32)]
        Counter,

        #[persistent]
        #[value(String)]
        Message { sender: Address },

        #[temporary]
        #[value(Address)]
        LastCaller { timestamp: u64 },

        #[persistent]
        #[value(bool)]
        Flag { key: String, owner: Address },

        #[persistent]
        #[value(Option<String>)]
        OptionalMessage { id: u32 },

        #[temporary]
        #[status]
        TempStatus { id: u32 },

        #[persistent]
        #[status]
        PersistentStatus { id: u32 },
    }
}

#[contractimpl]
impl Contract {
    pub const fn __constructor() {}

    pub fn increment_counter(env: &Env) -> u32 {
        let current_counter = storage::try_counter(env).unwrap_or(0);
        let new_counter = current_counter + 1;
        storage::set_counter(env, &new_counter);
        new_counter
    }

    pub fn set_message(env: &Env, sender: Address, message: String) {
        storage::set_message(env, sender, &message);
    }

    pub fn message(env: &Env, sender: Address) -> Option<String> {
        storage::try_message(env, sender)
    }

    pub fn set_last_caller(env: &Env, timestamp: u64, caller: Address) {
        storage::set_last_caller(env, timestamp, &caller);
    }

    pub fn last_caller(env: &Env, timestamp: u64) -> Option<Address> {
        storage::try_last_caller(env, timestamp)
    }

    pub fn set_flag(env: &Env, key: String, owner: Address, value: bool) {
        storage::set_flag(env, key, owner, &value);
    }

    pub fn flag(env: &Env, key: String, owner: Address) -> Option<bool> {
        storage::try_flag(env, key, owner)
    }

    pub fn set_optional_message(env: &Env, id: u32, message: Option<String>) {
        storage::set_optional_message(env, id, &message);
    }

    pub fn optional_message(env: &Env, id: u32) -> Option<Option<String>> {
        storage::try_optional_message(env, id)
    }

    pub fn message_required(env: &Env, sender: Address) -> String {
        storage::message(env, sender)
    }

    pub fn flag_required(env: &Env, key: String, owner: Address) -> bool {
        storage::flag(env, key, owner)
    }

    pub fn set_temp_status(env: &Env, id: u32) {
        storage::set_temp_status_status(env, id);
    }

    pub fn is_temp_status(env: &Env, id: u32) -> bool {
        storage::is_temp_status(env, id)
    }

    pub fn set_persistent_status(env: &Env, id: u32) {
        storage::set_persistent_status_status(env, id);
    }

    pub fn is_persistent_status(env: &Env, id: u32) -> bool {
        storage::is_persistent_status(env, id)
    }
}

#[test]
fn instance_storage_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.increment_counter(), 1);
    assert_eq!(client.increment_counter(), 2);
}

#[test]
fn persistent_storage_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let message = String::from_str(&env, "Hello, Soroban!");

    assert_eq!(client.message(&sender), None);

    client.set_message(&sender, &message);
    assert_eq!(client.message(&sender), Some(message));
}

#[test]
fn temporary_storage_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let timestamp = 12345u64;
    let caller = Address::generate(&env);

    assert_eq!(client.last_caller(&timestamp), None);

    client.set_last_caller(&timestamp, &caller);
    assert_eq!(client.last_caller(&timestamp), Some(caller));
}

#[test]
fn multiple_fields_storage_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let key = String::from_str(&env, "test_key");
    let owner = Address::generate(&env);

    assert_eq!(client.flag(&key, &owner), None);

    client.set_flag(&key, &owner, &true);
    assert_eq!(client.flag(&key, &owner), Some(true));

    client.set_flag(&key, &owner, &false);
    assert_eq!(client.flag(&key, &owner), Some(false));
}

#[test]
fn storage_mapping_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sender1 = Address::generate(&env);
    let sender2 = Address::generate(&env);
    let sender3 = Address::generate(&env);
    let message1 = String::from_str(&env, "Message 1");
    let message2 = String::from_str(&env, "Message 2");
    let message3 = String::from_str(&env, "Message 3");

    client.set_message(&sender1, &message1);
    client.set_message(&sender2, &message2);
    client.set_message(&sender3, &message3);

    assert_eq!(client.message(&sender1), Some(message1));
    assert_eq!(client.message(&sender2), Some(message2));
    assert_eq!(client.message(&sender3), Some(message3));
}

#[test]
fn persistent_storage_overwrite_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let message1 = String::from_str(&env, "First message");
    let message2 = String::from_str(&env, "Second message");

    client.set_message(&sender, &message1);
    assert_eq!(client.message(&sender), Some(message1));

    client.set_message(&sender, &message2);
    assert_eq!(client.message(&sender), Some(message2));
}

#[test]
fn empty_string_keys_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let empty_key = String::from_str(&env, "");
    let owner = Address::generate(&env);

    client.set_flag(&empty_key, &owner, &true);
    assert_eq!(client.flag(&empty_key, &owner), Some(true));
}

#[test]
fn all_storage_types_interaction_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let counter = client.increment_counter();

    let sender = Address::generate(&env);
    let message = String::from_str(&env, "Test message");
    client.set_message(&sender, &message);

    let timestamp = 12345u64;
    let caller = Address::generate(&env);
    client.set_last_caller(&timestamp, &caller);

    let key = String::from_str(&env, "test");
    let owner = Address::generate(&env);
    client.set_flag(&key, &owner, &true);

    assert_eq!(client.increment_counter(), counter + 1);
    assert_eq!(client.message(&sender), Some(message));
    assert_eq!(client.last_caller(&timestamp), Some(caller));
    assert_eq!(client.flag(&key, &owner), Some(true));
}

#[test]
fn optional_value_storage_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let id = 1u32;

    assert_eq!(client.optional_message(&id), None);

    let message = String::from_str(&env, "Optional message");
    client.set_optional_message(&id, &Some(message.clone()));
    assert_eq!(client.optional_message(&id), Some(Some(message)));

    client.set_optional_message(&id, &None);
    assert_eq!(client.optional_message(&id), None);
}

#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn required_getter_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    client.message_required(&sender);
}

#[test]
fn required_getter_succeeds_when_set() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let message = String::from_str(&env, "Required message");

    client.set_message(&sender, &message);
    assert_eq!(client.message_required(&sender), message);
}

#[test]
fn temporary_status_storage_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let id = 1u32;

    assert!(!client.is_temp_status(&id));

    client.set_temp_status(&id);
    assert!(client.is_temp_status(&id));
}

#[test]
fn test_persistent_status() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let id = 42u32;

    assert!(!client.is_persistent_status(&id));

    client.set_persistent_status(&id);
    assert!(client.is_persistent_status(&id));

    let other_id = 43u32;
    assert!(!client.is_persistent_status(&other_id));
}
