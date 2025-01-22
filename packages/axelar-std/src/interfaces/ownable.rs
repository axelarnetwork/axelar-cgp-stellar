use core::fmt::Debug;

use soroban_sdk::{contractclient, Address, Env, IntoVal};
use stellar_axelar_std_derive::IntoEvent;

use crate::events::Event;
use crate::interfaces::storage;

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
        .get(&storage::owner::DataKey::Interfaces_Owner)
        .expect("owner must be set during contract construction")
}

/// Default implementation of the [OwnableInterface] trait. Ensures the current owner is authorized and emits an event after the transfer.
pub fn transfer_ownership<T: OwnableInterface>(env: &Env, new_owner: Address) {
    let current_owner = T::owner(env);
    current_owner.require_auth();

    set_owner(env, &new_owner);

    OwnershipTransferredEvent {
        previous_owner: current_owner,
        new_owner,
    }
    .emit(env);
}

/// Default implementation accompanying the [OwnableInterface] trait. This should never be part of a contract interface,
/// but allows contracts internally to set the owner.
pub fn set_owner(env: &Env, owner: &Address) {
    env.storage()
        .instance()
        .set(&storage::owner::DataKey::Interfaces_Owner, owner);
}

#[derive(Clone, Debug, PartialEq, Eq, IntoEvent)]
pub struct OwnershipTransferredEvent {
    pub previous_owner: Address,
    pub new_owner: Address,
}

#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    use crate::interfaces::testdata::Contract;
    use crate::interfaces::{OwnableClient, OwnershipTransferredEvent};
    use crate::{assert_auth, assert_auth_err, events};

    fn prepare_client(env: &Env, owner: Option<Address>) -> OwnableClient {
        let operator = Address::generate(env);
        let contract_id = env.register(Contract, (owner, operator));
        OwnableClient::new(env, &contract_id)
    }

    #[test]
    fn owner_fails_if_owner_not_set() {
        let env = Env::default();
        let client = prepare_client(&env, None);

        assert!(client.try_owner().is_err());
    }

    #[test]
    fn owner_returns_correct_owner_when_set() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let client = prepare_client(&env, Some(owner.clone()));

        assert_eq!(client.owner(), owner);
    }

    #[test]
    fn transfer_ownership_fails_if_caller_is_not_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let client = prepare_client(&env, Some(owner));

        let new_owner = Address::generate(&env);
        assert_auth_err!(new_owner, client.transfer_ownership(&new_owner));
    }

    #[test]
    fn transfer_ownership_succeeds_if_caller_is_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let client = prepare_client(&env, Some(owner.clone()));

        assert_eq!(client.owner(), owner);

        let new_owner = Address::generate(&env);
        assert_auth!(owner, client.transfer_ownership(&new_owner));

        goldie::assert!(events::fmt_last_emitted_event::<OwnershipTransferredEvent>(
            &env
        ));

        assert_eq!(client.owner(), new_owner);
    }
}
