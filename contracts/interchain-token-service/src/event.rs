use core::fmt::Debug;

use stellar_axelar_soroban_std::events::Event;
use soroban_sdk::{Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Topics, Val};

#[derive(Debug, PartialEq, Eq)]
pub struct TrustedChainSetEvent {
    pub chain: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TrustedChainRemovedEvent {
    pub chain: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FlowLimitSetEvent {
    pub token_id: BytesN<32>,
    /// A `None` value implies that flow limit checks have been disabled for this `token_id`
    pub flow_limit: Option<i128>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterchainTokenDeployedEvent {
    pub token_id: BytesN<32>,
    pub token_address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub minter: Option<Address>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterchainTokenDeploymentStartedEvent {
    pub token_id: BytesN<32>,
    pub token_address: Address,
    pub destination_chain: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub minter: Option<Address>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterchainTokenIdClaimedEvent {
    pub token_id: BytesN<32>,
    pub deployer: Address,
    pub salt: BytesN<32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterchainTransferSentEvent {
    pub token_id: BytesN<32>,
    pub source_address: Address,
    pub destination_chain: String,
    pub destination_address: Bytes,
    pub amount: i128,
    pub data: Option<Bytes>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterchainTransferReceivedEvent {
    pub source_chain: String,
    pub token_id: BytesN<32>,
    pub source_address: Bytes,
    pub destination_address: Address,
    pub amount: i128,
    pub data: Option<Bytes>,
}

impl Event for TrustedChainSetEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (Symbol::new(env, "trusted_chain_set"), self.chain.to_val())
    }
}

impl Event for TrustedChainRemovedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "trusted_chain_removed"),
            self.chain.to_val(),
        )
    }
}

impl Event for FlowLimitSetEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "flow_limit_set"),
            self.token_id.to_val(),
            self.flow_limit,
        )
    }
}

impl Event for InterchainTokenDeployedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "interchain_token_deployed"),
            self.token_id.to_val(),
            self.token_address.to_val(),
            self.name.to_val(),
            self.symbol.to_val(),
            self.decimals,
            self.minter.clone(),
        )
    }
}

impl Event for InterchainTokenDeploymentStartedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "token_deployment_started"),
            self.token_id.to_val(),
            self.token_address.to_val(),
            self.destination_chain.to_val(),
            self.name.to_val(),
            self.symbol.to_val(),
            self.decimals,
            self.minter.clone(),
        )
    }
}

impl Event for InterchainTokenIdClaimedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "interchain_token_id_claimed"),
            self.token_id.to_val(),
            self.deployer.to_val(),
            self.salt.to_val(),
        )
    }
}

impl Event for InterchainTransferSentEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "interchain_transfer_sent"),
            self.token_id.to_val(),
            self.source_address.to_val(),
            self.destination_chain.to_val(),
            self.destination_address.to_val(),
            self.amount,
        )
    }

    fn data(&self, _env: &Env) -> impl IntoVal<Env, Val> + Debug {
        (self.data.clone(),)
    }
}

impl Event for InterchainTransferReceivedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "interchain_transfer_received"),
            self.source_chain.to_val(),
            self.token_id.to_val(),
            self.source_address.to_val(),
            self.destination_address.to_val(),
            self.amount,
        )
    }

    fn data(&self, _env: &Env) -> impl IntoVal<Env, Val> + Debug {
        (self.data.clone(),)
    }
}

#[cfg(any(test, feature = "testutils"))]
use stellar_axelar_soroban_std::impl_event_testutils;

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(TrustedChainSetEvent, (Symbol, String), ());

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(TrustedChainRemovedEvent, (Symbol, String), ());

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(FlowLimitSetEvent, (Symbol, BytesN<32>, Option<i128>), ());

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    InterchainTokenDeployedEvent,
    (
        Symbol,
        BytesN<32>,
        Address,
        String,
        String,
        u32,
        Option<Address>
    ),
    ()
);

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    InterchainTokenDeploymentStartedEvent,
    (
        Symbol,
        BytesN<32>,
        Address,
        String,
        String,
        String,
        u32,
        Option<Address>
    ),
    ()
);

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    InterchainTokenIdClaimedEvent,
    (Symbol, BytesN<32>, Address, BytesN<32>),
    ()
);

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    InterchainTransferSentEvent,
    (Symbol, BytesN<32>, Address, String, Bytes, i128),
    (Option<Bytes>)
);

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    InterchainTransferReceivedEvent,
    (Symbol, String, BytesN<32>, Bytes, Address, i128),
    (Option<Bytes>)
);
