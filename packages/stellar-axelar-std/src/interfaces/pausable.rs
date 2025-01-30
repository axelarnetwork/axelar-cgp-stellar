use core::fmt::Debug;

use soroban_sdk::{contractclient, Env};

use crate as stellar_axelar_std;
use crate::events::Event;
use crate::interfaces::{storage, OwnableInterface};
use crate::IntoEvent;

#[contractclient(name = "PausableClient")]
pub trait PausableInterface: OwnableInterface {
    /// Returns whether the contract is currently paused.
    fn paused(env: &Env) -> bool;

    /// Pauses the contract. Only callable by the owner.
    fn pause(env: &Env);

    /// Unpauses the contract. Only callable by the owner.
    fn unpause(env: &Env);
}

/// Default implementation of the [`PausableInterface`] trait.
pub fn paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .has(&storage::pausable::DataKey::Interfaces_Paused)
}

/// Default implementation of the [`PausableInterface`] trait.
pub fn pause<T: PausableInterface>(env: &Env) {
    T::owner(env).require_auth();

    env.storage()
        .instance()
        .set(&storage::pausable::DataKey::Interfaces_Paused, &());

    PausedEvent {}.emit(env);
}

/// Default implementation of the [`PausableInterface`] trait.
pub fn unpause<T: PausableInterface>(env: &Env) {
    T::owner(env).require_auth();

    env.storage()
        .instance()
        .remove(&storage::pausable::DataKey::Interfaces_Paused);

    UnpausedEvent {}.emit(env);
}

#[derive(Clone, Debug, PartialEq, Eq, IntoEvent)]
pub struct PausedEvent {}

#[derive(Clone, Debug, PartialEq, Eq, IntoEvent)]
pub struct UnpausedEvent {}

#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};

    use super::{PausedEvent, UnpausedEvent};
    use crate as stellar_axelar_std;
    use crate::interfaces::{OwnableInterface, PausableInterface};
    use crate::{assert_auth, assert_auth_err, assert_contract_err, events, when_not_paused};

    #[contracterror]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ContractError {
        ContractPaused = 1,
    }

    #[contract]
    pub struct Contract;

    #[contractimpl]
    impl Contract {
        pub fn __constructor(env: &Env, owner: Address) {
            crate::interfaces::ownable::set_owner(env, &owner);
        }
    }

    #[contractimpl]
    impl PausableInterface for Contract {
        fn paused(env: &Env) -> bool {
            super::paused(env)
        }

        fn pause(env: &Env) {
            super::pause::<Self>(env)
        }

        fn unpause(env: &Env) {
            super::unpause::<Self>(env)
        }
    }

    #[contractimpl]
    impl OwnableInterface for Contract {
        fn owner(env: &Env) -> Address {
            crate::interfaces::ownable::owner(env)
        }

        fn transfer_ownership(env: &Env, new_owner: Address) {
            crate::interfaces::ownable::transfer_ownership::<Self>(env, new_owner)
        }
    }

    #[contractimpl]
    impl Contract {
        #[when_not_paused]
        pub fn test(env: &Env) -> Result<u32, ContractError> {
            Ok(42)
        }
    }

    fn setup<'a>() -> (Env, ContractClient<'a>) {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner,));
        let client = ContractClient::new(&env, &contract_id);
        (env, client)
    }

    #[test]
    fn paused_returns_false_by_default() {
        let (_, client) = setup();
        assert!(!client.paused());
    }

    #[test]
    fn pause_succeeds_when_not_paused() {
        let (env, client) = setup();

        assert!(!client.paused());

        assert_auth!(client.owner(), client.pause());
        goldie::assert!(events::fmt_last_emitted_event::<PausedEvent>(&env));

        assert!(client.paused());
    }

    #[test]
    fn pause_succeeds_when_already_paused() {
        let (_, client) = setup();

        assert_auth!(client.owner(), client.pause());
        assert_auth!(client.owner(), client.pause());
        assert!(client.paused());
    }

    #[test]
    fn pause_fails_when_not_owner() {
        let (env, client) = setup();

        assert_auth_err!(Address::generate(&env), client.pause());
    }

    #[test]
    fn unpause_succeeds_when_paused() {
        let (env, client) = setup();

        assert_auth!(client.owner(), client.pause());
        assert!(client.paused());

        assert_auth!(client.owner(), client.unpause());
        goldie::assert!(events::fmt_last_emitted_event::<UnpausedEvent>(&env));

        assert!(!client.paused());
    }

    #[test]
    fn unpause_fails_when_not_paused() {
        let (_, client) = setup();

        assert_auth!(client.owner(), client.unpause());
        assert!(!client.paused());
    }

    #[test]
    fn unpause_fails_when_not_owner() {
        let (env, client) = setup();

        assert_auth_err!(Address::generate(&env), client.unpause());
    }

    #[test]
    fn test_succeeds_when_not_paused() {
        let (_, client) = setup();
        assert_eq!(client.test(), 42);
    }

    #[test]
    fn test_fails_when_paused() {
        let (_, client) = setup();

        client.mock_all_auths().pause();
        assert_contract_err!(client.try_test(), ContractError::ContractPaused);
    }
}
