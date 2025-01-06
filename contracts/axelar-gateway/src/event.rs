use crate::types::Message;

use core::fmt::Debug;

use axelar_soroban_std::events::Event;
use soroban_sdk::{Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Topics, Val, Vec};

#[derive(Debug, PartialEq, Eq)]
pub struct ContractCalledEvent {
    pub caller: Address,
    pub destination_chain: String,
    pub destination_address: String,
    pub payload: Bytes,
    pub payload_hash: BytesN<32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MessageApprovedEvent {
    pub message: Message,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MessageExecutedEvent {
    pub message: Message,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SignersRotatedEvent {
    pub epoch: u64,
    pub signers_hash: BytesN<32>,
}

impl Event for ContractCalledEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "contract_called"),
            self.caller.to_val(),
            self.destination_chain.to_val(),
            self.destination_address.to_val(),
            self.payload_hash.to_val(),
        )
    }

    fn data(&self, _env: &Env) -> impl IntoVal<Env, Val> + Debug {
        (self.payload.clone(),)
    }
}

impl Event for MessageApprovedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "message_approved"),
            self.message.clone(),
        )
    }

    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        Vec::<Val>::new(env)
    }
}

impl Event for MessageExecutedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "message_executed"),
            self.message.clone(),
        )
    }

    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        Vec::<Val>::new(env)
    }
}

impl Event for SignersRotatedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "signers_rotated"),
            self.epoch,
            self.signers_hash.to_val(),
        )
    }

    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        Vec::<Val>::new(env)
    }
}

#[cfg(any(test, feature = "testutils"))]
use axelar_soroban_std::impl_event_testutils;


#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    ContractCalledEvent,
    (Symbol, Address, String, String, BytesN<32>),
    (Bytes)
);

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    MessageApprovedEvent,
    (Symbol, Message),
    ()
);

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    MessageExecutedEvent,
    (Symbol, Message),
    ()
);

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    SignersRotatedEvent,
    (Symbol, u64, BytesN<32>),
    ()
);
