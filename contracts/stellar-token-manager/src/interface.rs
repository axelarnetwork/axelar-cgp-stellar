use soroban_sdk::{contractclient, Address, Env, Symbol, Val, Vec};
use stellar_axelar_std::interfaces::{OwnableInterface, UpgradableInterface};

use crate::error::ContractError;

#[contractclient(name = "TokenManagerClient")]
pub trait TokenManagerInterface: OwnableInterface + UpgradableInterface {
    /// Executes a function on the given contract.
    ///
    /// # Arguments
    /// * `contract` - The address of the contract to execute the function on.
    /// * `func` - The symbol of the function to execute.
    /// * `args` - The arguments to pass to the function.
    ///
    /// # Returns
    /// - `Ok(Val)` - The result of the function execution.
    ///
    /// # Authorization
    ///  - [`OwnableInterface::owner`] must have authorized.
    fn execute(
        env: &Env,
        contract: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> Result<Val, ContractError>;
}
