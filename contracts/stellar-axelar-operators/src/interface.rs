use soroban_sdk::{contractclient, Address, Env, Symbol, Val, Vec};
use stellar_axelar_std::interfaces::OwnableInterface;

use crate::error::ContractError;

#[contractclient(name = "AxelarOperatorsClient")]
pub trait AxelarOperatorsInterface: OwnableInterface {
    /// Return whether specified account is an operator.
    fn is_operator(env: Env, account: Address) -> bool;

    /// Add an address as an operator.
    ///
    /// The operator is authorized to execute any third party contract via this contract.
    /// An app can give a privileged role to this contract, which can then allow multiple operators
    /// to call it, e.g. `refund` on the gas service.
    ///
    /// # Arguments
    /// * `account` - The address to be added as an operator.
    ///
    /// # Errors
    /// - [`ContractError::OperatorAlreadyAdded`]: If the specified account is already an operator.
    ///
    /// # Authorization
    /// - [`OwnableInterface::owner`] must authorize.
    fn add_operator(env: Env, account: Address) -> Result<(), ContractError>;

    /// Remove an address as an operator.
    ///
    /// The address is no longer authorized to execute apps via this contract.
    ///
    /// # Arguments
    /// * `account` - The address to be removed as an operator.
    ///
    /// # Errors
    /// - [`ContractError::NotAnOperator`]: If the specified account is not an operator.
    ///
    /// # Authorization
    ///  - [`OwnableInterface::owner`] must authorize.
    fn remove_operator(env: Env, account: Address) -> Result<(), ContractError>;

    /// Execute a function on any contract as the operators contract.
    ///
    /// # Arguments
    /// * `operator` - The address of the operator executing the function.
    /// * `contract` - The address of the contract on which the function will be executed.
    /// * `func` - The symbol representing the function to be executed.
    /// * `args` - The arguments to be passed to the function.
    ///
    /// # Returns
    /// - `Ok(Val)`: Returns the result of the function execution.
    ///
    /// # Errors
    /// - [`ContractError::NotAnOperator`]: If the specified operator is not authorized.
    ///
    /// # Authorization
    /// - An `operator` must authorize.
    fn execute(
        env: Env,
        operator: Address,
        contract: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> Result<Val, ContractError>;
}
