use soroban_sdk::{contractclient, Address, Env, Symbol, Val, Vec};

use crate::error::ContractError;

#[allow(dead_code)]
#[contractclient(name = "AxelarOperatorsClient")]
pub trait AxelarOperatorsInterface {
    /// Return whether specified account is an operator.
    fn is_operator(env: Env, account: Address) -> bool;

    /// Add an address as an operator.
    ///
    /// The operator is authorized to execute specific functions (e.g. setting flow limits) for a contract.
    ///
    /// # Arguments
    /// * `account` - The address to be added as an operator.
    ///
    /// # Returns
    /// - `Ok(())`
    ///
    /// # Errors
    /// - `ContractError::OperatorAlreadyAdded`: If the specified account is already an operator.
    ///
    /// # Authorization
    /// - Must be called by [`Self::owner`].
    fn add_operator(env: Env, account: Address) -> Result<(), ContractError>;

    /// Remove an address as an operator.
    ///
    /// The address is no longer authorized to execute specific operator functions (e.g. setting flow limits) for a contract.
    ///
    /// # Arguments
    /// * `account` - The address to be removed as an operator.
    ///
    /// # Returns
    /// - `Ok(())`
    ///
    /// # Errors
    /// - `ContractError::NotAnOperator`: If the specified account is not an operator.
    ///
    /// # Authorization
    /// - Must be called by [`Self::owner`].
    fn remove_operator(env: Env, account: Address) -> Result<(), ContractError>;

    /// Execute a function on a contract as an operator.
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
    /// - `ContractError::NotAnOperator`: If the specified operator is not authorized.
    /// - Any error propagated from `env.invoke_contract`.
    ///
    /// # Authorization
    /// - The `operator` must authenticate.
    fn execute(
        env: Env,
        operator: Address,
        contract: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> Result<Val, ContractError>;
}
