use core::fmt::Debug;

use axelar_soroban_std::events::Event;
use axelar_soroban_std_derive::{Ownable, Upgradable};
use soroban_sdk::{
    contract, contracterror, contractimpl, Address, Env, IntoVal, Symbol, Topics, Val,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
}

#[contract]
#[derive(Ownable, Upgradable)]
#[migratable(with_type = ())]
pub struct Contract;

#[derive(Debug, PartialEq, Eq)]
struct MigratedEvent {}

impl Event for MigratedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (Symbol::new(env, "migrated"),)
    }

    fn data(&self, _env: &Env) -> impl IntoVal<Env, Val> + Debug {}
}

#[contractimpl]
impl Contract {
    pub fn __constructor(env: &Env, owner: Address) {
        axelar_soroban_std::interfaces::set_owner(env, &owner);
    }
}

impl Contract {
    fn run_migration(env: &Env, _migration_data: ()) {
        MigratedEvent {}.emit(env);
    }
}
