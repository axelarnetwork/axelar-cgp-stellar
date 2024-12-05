use crate::events::Event;
#[cfg(any(test, feature = "testutils"))]
use crate::impl_event_testutils;
use core::fmt::Debug;
use soroban_sdk::{contractclient, Address, Env, IntoVal, Symbol, Topics, Val, Vec};

#[contractclient(name = "OwnableClient")]
pub trait OwnableInterface {
    /// Returns the address of the contract's owner.
    fn owner(env: &Env) -> Address;

    /// Transfers ownership of the contract to a new address.
    fn transfer_ownership(env: &Env, new_owner: Address);
}

/// Default implementation of the [OwnableInterface] trait.
pub fn owner(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&storage::DataKey::Interfaces_Owner)
        .expect("owner must be set during contract construction")
}

/// Default implementation of the [OwnableInterface] trait. Ensures the current owner is authorized and emits an event after the transfer.
pub fn transfer_ownership<T: OwnableInterface>(env: &Env, new_owner: Address) {
    let previous_owner = T::owner(env);
    previous_owner.require_auth();

    set_owner(env, &new_owner);

    OwnerChangedEvent {
        previous_owner,
        new_owner,
    }
    .emit(env);
}

/// Default implementation accompanying the [OwnableInterface] trait. This should never be part of a contract interface,
/// but allows contracts internally to set the owner.
pub fn set_owner(env: &Env, owner: &Address) {
    env.storage()
        .instance()
        .set(&storage::DataKey::Interfaces_Owner, owner);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerChangedEvent {
    pub previous_owner: Address,
    pub new_owner: Address,
}

impl Event for OwnerChangedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "ownership_transferred"),
            self.previous_owner.to_val(),
            self.new_owner.to_val(),
        )
    }

    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        Vec::<Val>::new(env)
    }
}

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(OwnerChangedEvent, (Symbol, Address, Address), ());

// submodule to encapsulate the disabled linting
mod storage {
    // linting is disabled for the enum variant names on purpose, so we can define names that would otherwise be invalid.
    // This way, if a contract that implements a shared interface defines a variant with the same name, the linter will
    // complain about it.
    #![allow(non_camel_case_types)]

    use soroban_sdk::contracttype;

    #[contracttype]
    /// Variants do not follow the naming convention of other variants to let the linter help to avoid
    /// collisions with contract types defined in other contracts that implement a shared interface.
    pub enum DataKey {
        Interfaces_Owner,
    }
}

#[cfg(test)]
mod test {
    use crate::interfaces::testdata::contract::Contract;
    use crate::interfaces::{OwnableClient, OwnerChangedEvent};
    use crate::{assert_invoke_auth_err, assert_invoke_auth_ok, events};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[test]
    fn owner_fails_if_owner_not_set() {
        let env = Env::default();
        let contract_id = env.register(Contract, (None::<Address>,));

        assert!(OwnableClient::new(&env, &contract_id).try_owner().is_err());
    }

    #[test]
    fn owner_returns_correct_owner_when_set() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner.clone(),));

        assert_eq!(OwnableClient::new(&env, &contract_id).owner(), owner);
    }

    #[test]
    fn transfer_ownership_fails_if_caller_is_not_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner.clone(),));

        let client = OwnableClient::new(&env, &contract_id);
        assert_eq!(client.owner(), owner);

        let new_owner = Address::generate(&env);
        assert_invoke_auth_err!(new_owner, client.try_transfer_ownership(&new_owner));
    }

    #[test]
    fn transfer_ownership_succeeds_if_caller_is_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner.clone(),));

        let client = OwnableClient::new(&env, &contract_id);
        assert_eq!(client.owner(), owner);

        let new_owner = Address::generate(&env);
        assert_invoke_auth_ok!(owner, client.try_transfer_ownership(&new_owner));

        goldie::assert!(events::fmt_last_emitted_event::<OwnerChangedEvent>(&env));

        assert_eq!(client.owner(), new_owner);
    }
}
