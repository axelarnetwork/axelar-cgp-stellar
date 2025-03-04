use soroban_sdk::token::{self, StellarAssetInterface};
use soroban_sdk::{contractclient, Address, BytesN, Env};
use stellar_axelar_std::interfaces::OwnableInterface;

use crate::error::ContractError;

#[contractclient(name = "InterchainTokenClient")]
pub trait InterchainTokenInterface:
    token::Interface + StellarAssetInterface + OwnableInterface
{
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
    /// # Errors
    /// - [`ContractError::NotMinter`]: If the specified minter is not authorized to mint tokens.
    /// - [`ContractError::InvalidAmount`]: If the specified amount is invalid (e.g. negative).
    ///
    /// # Authorization
    /// - The `minter` must authorize.
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
    /// - [`OwnableInterface::owner`] must authorize.
    fn add_minter(env: &Env, minter: Address);

    /// Removes a new minter from the Interchain Token contract.
    ///
    /// # Arguments
    /// * `minter` - The address to be added as a minter.
    ///
    /// # Authorization
    /// - [`OwnableInterface::owner`] must authorize.
    fn remove_minter(env: &Env, minter: Address);
}
