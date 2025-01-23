use soroban_sdk::token::{self, StellarAssetInterface};
use soroban_sdk::{contractclient, Address, BytesN, Env};

use crate::error::ContractError;

#[allow(dead_code)]
#[contractclient(name = "InterchainTokenClient")]
pub trait InterchainTokenInterface: token::Interface + StellarAssetInterface {
    /// Returns the Interchain Token ID
    fn token_id(env: &Env) -> BytesN<32>;

    /// Returns if the specified address is a minter.
    fn is_minter(env: &Env, minter: Address) -> bool;

    /// Mints new tokens from a specified minter to a specified address.
    ///
    /// # Arguments
    /// * `minter` - The address of the minter.
    /// * `to` - The address to which the tokens will be minted.
    /// * `amount` - The amount of tokens to be minted.
    ///
    /// # Returns
    /// - `Ok(())`
    ///
    /// # Errors
    /// - `ContractError::NotMinter`: If the specified minter is not authorized to mint tokens.
    /// - `ContractError::InvalidAmount`: If the specified amount is invalid (e.g., negative).
    ///
    /// # Authorization
    /// - The `minter` address must authenticate.
    fn mint_from(
        env: &Env,
        minter: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError>;

    /// Adds a new minter to the Interchain Token contract.
    ///
    /// # Arguments
    /// * `minter` - The address to be added as a minter.
    ///
    /// # Authorization
    /// - Must be called by [`Self::owner`].
    fn add_minter(env: &Env, minter: Address);

    /// Removes a new minter from the Interchain Token contract.
    ///
    /// # Arguments
    /// * `minter` - The address to be added as a minter.
    ///
    /// # Authorization
    /// - Must be called by [`Self::owner`].
    fn remove_minter(env: &Env, minter: Address);
}
