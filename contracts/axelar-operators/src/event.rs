use soroban_sdk::Address;

#[derive(Debug, PartialEq, Eq)]
pub struct OperatorAddedEvent {
    pub operator: Address,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OperatorRemovedEvent {
    pub operator: Address,
}
