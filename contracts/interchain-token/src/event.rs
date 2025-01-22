use soroban_sdk::Address;
use stellar_axelar_std::IntoEvent;

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct MinterAddedEvent {
    pub minter: Address,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct MinterRemovedEvent {
    pub minter: Address,
}
