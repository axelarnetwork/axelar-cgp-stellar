use soroban_sdk::Address;
use stellar_axelar_std::IntoEvent;

#[derive(Debug, PartialEq, IntoEvent)]
pub struct OperatorAddedEvent {
    pub operator: Address,
}

#[derive(Debug, PartialEq, IntoEvent)]
pub struct OperatorRemovedEvent {
    pub operator: Address,
}
