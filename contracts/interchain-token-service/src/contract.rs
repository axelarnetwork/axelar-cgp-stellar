use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::{executable::AxelarExecutableInterface, AxelarGatewayMessagingClient};
use axelar_soroban_std::events::Event;
use axelar_soroban_std::token::validate_token_metadata;
use axelar_soroban_std::ttl::{extend_instance_ttl, extend_persistent_ttl};
use axelar_soroban_std::{
    address::AddressExt, ensure, interfaces, types::Token, Operatable, Ownable, Upgradable,
};
use interchain_token::InterchainTokenClient;
use soroban_sdk::token::{self, StellarAssetClient};
use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;

use crate::abi::{get_message_type, MessageType as EncodedMessageType};
use crate::error::ContractError;
use crate::event::{
    InterchainTokenDeployedEvent, InterchainTokenDeploymentStartedEvent,
    InterchainTokenIdClaimedEvent, InterchainTransferReceivedEvent, InterchainTransferSentEvent,
    TrustedChainRemovedEvent, TrustedChainSetEvent,
};
use crate::executable::InterchainTokenExecutableClient;
use crate::interface::InterchainTokenServiceInterface;
use crate::storage_types::{DataKey, TokenIdConfigValue};
use crate::types::{
    DeployInterchainToken, HubMessage, InterchainTransfer, Message, TokenManagerType,
};
use crate::{flow_limit, token_handler};

const ITS_HUB_CHAIN_NAME: &str = "axelar";
const PREFIX_INTERCHAIN_TOKEN_ID: &str = "its-interchain-token-id";
const PREFIX_INTERCHAIN_TOKEN_SALT: &str = "interchain-token-salt";
const PREFIX_CANONICAL_TOKEN_SALT: &str = "canonical-token-salt";

#[contract]
#[derive(Operatable, Ownable, Upgradable)]
pub struct InterchainTokenService;

#[contractimpl]
impl InterchainTokenService {
    pub fn __constructor(
        env: Env,
        owner: Address,
        operator: Address,
        gateway: Address,
        gas_service: Address,
        its_hub_address: String,
        chain_name: String,
        interchain_token_wasm_hash: BytesN<32>,
    ) {
        interfaces::set_owner(&env, &owner);
        interfaces::set_operator(&env, &operator);
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
        env.storage()
            .instance()
            .set(&DataKey::ItsHubAddress, &its_hub_address);
        env.storage()
            .instance()
            .set(&DataKey::ChainName, &chain_name);
        env.storage().instance().set(
            &DataKey::InterchainTokenWasmHash,
            &interchain_token_wasm_hash,
        );
    }
}

#[contractimpl]
impl InterchainTokenServiceInterface for InterchainTokenService {
    fn chain_name(env: &Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::ChainName)
            .expect("chain name not found")
    }

    fn gas_service(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::GasService)
            .expect("gas service not found")
    }

    fn interchain_token_wasm_hash(env: &Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::InterchainTokenWasmHash)
            .expect("interchain token wasm hash not found")
    }

    fn its_hub_address(env: &Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::ItsHubAddress)
            .expect("its hub address not found")
    }

    fn its_hub_chain_name(env: &Env) -> String {
        String::from_str(env, ITS_HUB_CHAIN_NAME)
    }

    fn is_trusted_chain(env: &Env, chain: String) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::TrustedChain(chain))
    }

    fn set_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError> {
        Self::owner(env).require_auth();

        let key = DataKey::TrustedChain(chain.clone());

        ensure!(
            !env.storage().persistent().has(&key),
            ContractError::TrustedChainAlreadySet
        );

        env.storage().persistent().set(&key, &());

        TrustedChainSetEvent { chain }.emit(env);

        Ok(())
    }

    fn remove_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError> {
        Self::owner(env).require_auth();

        let key = DataKey::TrustedChain(chain.clone());

        ensure!(
            env.storage().persistent().has(&key),
            ContractError::TrustedChainNotSet
        );

        env.storage().persistent().remove(&key);

        TrustedChainRemovedEvent { chain }.emit(env);

        Ok(())
    }

    fn interchain_token_deploy_salt(env: &Env, deployer: Address, salt: BytesN<32>) -> BytesN<32> {
        let chain_name_hash = Self::chain_name_hash(env);
        env.crypto()
            .keccak256(
                &(
                    PREFIX_INTERCHAIN_TOKEN_SALT,
                    chain_name_hash,
                    deployer,
                    salt,
                )
                    .to_xdr(env),
            )
            .into()
    }

    fn interchain_token_id(env: &Env, sender: Address, salt: BytesN<32>) -> BytesN<32> {
        env.crypto()
            .keccak256(&(PREFIX_INTERCHAIN_TOKEN_ID, sender, salt).to_xdr(env))
            .into()
    }

    fn flow_limit(env: &Env, token_id: BytesN<32>) -> Option<i128> {
        flow_limit::flow_limit(env, token_id)
    }

    fn set_flow_limit(
        env: &Env,
        token_id: BytesN<32>,
        flow_limit: Option<i128>,
    ) -> Result<(), ContractError> {
        Self::operator(env).require_auth();

        flow_limit::set_flow_limit(env, token_id, flow_limit)
    }

    fn flow_out_amount(env: &Env, token_id: BytesN<32>) -> i128 {
        flow_limit::flow_out_amount(env, token_id)
    }

    fn flow_in_amount(env: &Env, token_id: BytesN<32>) -> i128 {
        flow_limit::flow_in_amount(env, token_id)
    }

    /// Computes a 32-byte deployment salt for a canonical token using the provided token address.
    ///
    /// The salt is derived by hashing a combination of a prefix, the chain name hash,
    /// and the token address. This ensures uniqueness and consistency for the deployment
    /// of canonical tokens across chains.
    ///
    /// # Parameters
    /// - `env`: A reference to the current environment, used for accessing chain-specific
    ///   utilities such as cryptographic functions.
    /// - `token_address`: The address of the token for which the deployment salt is being generated.
    ///
    /// # Returns
    /// - A `BytesN<32>` value representing the computed deployment salt.
    fn canonical_token_deploy_salt(env: &Env, token_address: Address) -> BytesN<32> {
        let chain_name_hash = Self::chain_name_hash(env);
        env.crypto()
            .keccak256(&(PREFIX_CANONICAL_TOKEN_SALT, chain_name_hash, token_address).to_xdr(env))
            .into()
    }

    /// Retrieves the address of the token associated with the specified token ID.
    fn token_address(env: &Env, token_id: BytesN<32>) -> Address {
        Self::token_id_config(env, token_id)
            .expect("token id config not found")
            .token_address
    }

    /// Retrieves the type of the token manager type associated with the specified token ID.
    fn token_manager_type(env: &Env, token_id: BytesN<32>) -> TokenManagerType {
        Self::token_id_config(env, token_id)
            .expect("token id config not found")
            .token_manager_type
    }

    fn deploy_interchain_token(
        env: &Env,
        caller: Address,
        salt: BytesN<32>,
        token_metadata: TokenMetadata,
        initial_supply: i128,
        minter: Option<Address>,
    ) -> Result<BytesN<32>, ContractError> {
        caller.require_auth();

        let initial_minter = if initial_supply > 0 {
            Some(env.current_contract_address())
        } else if let Some(ref minter) = minter {
            ensure!(
                *minter != env.current_contract_address(),
                ContractError::InvalidMinter
            );
            Some(minter.clone())
        } else {
            None
        };

        let deploy_salt = Self::interchain_token_deploy_salt(env, caller.clone(), salt);
        let token_id = Self::interchain_token_id(env, Address::zero(env), deploy_salt);

        let deployed_address = Self::deploy_interchain_token_contract(
            env,
            initial_minter,
            token_id.clone(),
            token_metadata,
        );

        if initial_supply > 0 {
            StellarAssetClient::new(env, &deployed_address).mint(&caller, &initial_supply);

            if let Some(minter) = minter {
                let token = InterchainTokenClient::new(env, &deployed_address);
                token.remove_minter(&env.current_contract_address());
                token.add_minter(&minter);
            }
        }

        Self::set_token_id_config(
            env,
            token_id.clone(),
            TokenIdConfigValue {
                token_address: deployed_address,
                token_manager_type: TokenManagerType::NativeInterchainToken,
            },
        );

        Ok(token_id)
    }

    /// Deploys an interchain token to a remote chain.
    ///
    /// This function initiates the deployment of an interchain token to a specified
    /// destination chain. It validates the token metadata, emits a deployment event,
    /// and triggers the necessary cross-chain call.
    ///
    /// # Arguments
    /// - `env`: Reference to the contract environment.
    /// - `caller`: Address of the caller initiating the deployment. The caller must authenticate.
    /// - `salt`: A 32-byte unique salt used for token deployment.
    /// - `destination_chain`: The name of the destination chain where the token will be deployed.
    /// - `gas_token`: The token used to pay for the gas cost of the cross-chain call.
    ///
    /// # Returns
    /// - `Result<BytesN<32>, ContractError>`: On success, returns the token ID (`BytesN<32>`).
    ///   On failure, returns a `ContractError`.
    ///
    /// # Errors
    /// - `ContractError::InvalidTokenId`: If the token ID does not exist in the persistent storage.
    /// - Any error propagated from `pay_gas_and_call_contract`.
    fn deploy_remote_interchain_token(
        env: &Env,
        caller: Address,
        salt: BytesN<32>,
        destination_chain: String,
        gas_token: Token,
    ) -> Result<BytesN<32>, ContractError> {
        caller.require_auth();

        let deploy_salt = Self::interchain_token_deploy_salt(env, caller.clone(), salt);

        Self::deploy_remote_token(env, caller, deploy_salt, destination_chain, gas_token)
    }

    /// Deploys a remote canonical token on a specified destination chain.
    ///
    /// This function computes a deployment salt and uses it to deploy a canonical
    /// representation of a token on the remote chain. It retrieves the token metadata
    /// from the token address and ensures the metadata is valid before initiating
    /// the deployment.
    ///
    /// # Arguments
    /// * `env` - Reference to the environment object.
    /// * `token_address` - The address of the token to be deployed.
    /// * `destination_chain` - The name of the destination chain where the token will be deployed.
    /// * `spender` - The spender of the cross-chain gas.
    /// * `gas_token` - The token used to pay for gas during the deployment.
    ///
    /// # Returns
    /// Returns the token ID of the deployed token on the remote chain, or an error if the deployment fails.
    ///
    /// # Errors
    /// Returns `ContractError` if the deployment fails or if token metadata is invalid.
    fn deploy_remote_canonical_token(
        env: &Env,
        token_address: Address,
        destination_chain: String,
        spender: Address,
        gas_token: Token,
    ) -> Result<BytesN<32>, ContractError> {
        let deploy_salt = Self::canonical_token_deploy_salt(env, token_address);

        let token_id =
            Self::deploy_remote_token(env, spender, deploy_salt, destination_chain, gas_token)?;

        Ok(token_id)
    }

    fn interchain_transfer(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        destination_address: Bytes,
        amount: i128,
        data: Option<Bytes>,
        gas_token: Token,
    ) -> Result<(), ContractError> {
        ensure!(amount > 0, ContractError::InvalidAmount);

        caller.require_auth();

        token_handler::take_token(
            env,
            &caller,
            Self::token_id_config_with_extended_ttl(env, token_id.clone())?,
            amount,
        )?;

        flow_limit::add_flow_out(env, token_id.clone(), amount)?;

        InterchainTransferSentEvent {
            token_id: token_id.clone(),
            source_address: caller.clone(),
            destination_chain: destination_chain.clone(),
            destination_address: destination_address.clone(),
            amount,
            data: data.clone(),
        }
        .emit(env);

        let message = Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: caller.clone().to_xdr(env),
            destination_address,
            amount,
            data,
        });

        Self::pay_gas_and_call_contract(env, caller, destination_chain, message, gas_token)?;

        Ok(())
    }

    /// Registers a canonical token as an interchain token.
    ///
    /// # Arguments
    /// * `env` - A reference to the environment in which the function operates.
    /// * `token_address` - The address of the canonical token.
    ///
    /// # Returns
    /// * `Result<BytesN<32>, ContractError>` - The token ID assigned to this canonical token if successful.
    ///
    /// # Errors
    /// * `ContractError::TokenAlreadyRegistered` - If the token ID is already registered.
    fn register_canonical_token(
        env: &Env,
        token_address: Address,
    ) -> Result<BytesN<32>, ContractError> {
        let deploy_salt = Self::canonical_token_deploy_salt(env, token_address.clone());
        let token_id = Self::interchain_token_id(env, Address::zero(env), deploy_salt.clone());

        ensure!(
            !env.storage()
                .persistent()
                .has(&DataKey::TokenIdConfigKey(token_id.clone())),
            ContractError::TokenAlreadyRegistered
        );

        InterchainTokenIdClaimedEvent {
            token_id: token_id.clone(),
            deployer: Address::zero(env),
            salt: deploy_salt,
        }
        .emit(env);

        Self::set_token_id_config(
            env,
            token_id.clone(),
            TokenIdConfigValue {
                token_address,
                token_manager_type: TokenManagerType::LockUnlock,
            },
        );

        Ok(token_id)
    }
}

#[contractimpl]
impl AxelarExecutableInterface for InterchainTokenService {
    fn gateway(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Gateway)
            .expect("gateway not found")
    }

    fn execute(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) {
        Self::validate_message(&env, &source_chain, &message_id, &source_address, &payload)
            .unwrap_or_else(|err| panic_with_error!(env, err));

        Self::execute_message(&env, source_chain, message_id, source_address, payload)
            .unwrap_or_else(|err| panic_with_error!(env, err));
    }
}

impl InterchainTokenService {
    // Modify this function to add migration logic
    const fn run_migration(_env: &Env, _migration_data: ()) {}

    fn pay_gas_and_call_contract(
        env: &Env,
        caller: Address,
        destination_chain: String,
        message: Message,
        gas_token: Token,
    ) -> Result<(), ContractError> {
        // Note: ITS Hub chain as the actual destination chain for the messsage isn't supported
        ensure!(
            Self::is_trusted_chain(env, destination_chain.clone()),
            ContractError::UntrustedChain
        );

        let gateway = AxelarGatewayMessagingClient::new(env, &Self::gateway(env));
        let gas_service = AxelarGasServiceClient::new(env, &Self::gas_service(env));

        let payload = HubMessage::SendToHub {
            destination_chain: destination_chain.clone(),
            message,
        }
        .abi_encode(env)?;

        let hub_chain = Self::its_hub_chain_name(env);
        let hub_address = Self::its_hub_address(env);

        gas_service.pay_gas(
            &env.current_contract_address(),
            &hub_chain,
            &hub_address,
            &payload,
            &caller,
            &gas_token,
            &Bytes::new(env),
        );

        gateway.call_contract(
            &env.current_contract_address(),
            &hub_chain,
            &hub_address,
            &payload,
        );

        extend_persistent_ttl(env, &DataKey::TrustedChain(destination_chain));
        extend_instance_ttl(env);

        Ok(())
    }

    fn execute_message(
        env: &Env,
        source_chain: String,
        message_id: String,
        _source_address: String,
        payload: Bytes,
    ) -> Result<(), ContractError> {
        let (source_chain, message) = Self::get_execute_params(env, source_chain, &payload)?;

        match message {
            Message::InterchainTransfer(InterchainTransfer {
                token_id,
                source_address,
                destination_address,
                amount,
                data,
            }) => {
                let destination_address = Address::from_xdr(env, &destination_address)
                    .map_err(|_| ContractError::InvalidDestinationAddress)?;

                let token_config_value =
                    Self::token_id_config_with_extended_ttl(env, token_id.clone())?;

                flow_limit::add_flow_in(env, token_id.clone(), amount)?;

                token_handler::give_token(
                    env,
                    &destination_address,
                    token_config_value.clone(),
                    amount,
                )?;

                InterchainTransferReceivedEvent {
                    source_chain: source_chain.clone(),
                    token_id: token_id.clone(),
                    source_address: source_address.clone(),
                    destination_address: destination_address.clone(),
                    amount,
                    data: data.clone(),
                }
                .emit(env);

                let token_address = token_config_value.token_address;

                if let Some(payload) = data {
                    let executable =
                        InterchainTokenExecutableClient::new(env, &destination_address);
                    executable.execute_with_interchain_token(
                        &source_chain,
                        &message_id,
                        &source_address,
                        &payload,
                        &token_id,
                        &token_address,
                        &amount,
                    );
                }
            }
            Message::DeployInterchainToken(DeployInterchainToken {
                token_id,
                name,
                symbol,
                decimals,
                minter,
            }) => {
                ensure!(
                    Self::token_id_config(env, token_id.clone()).is_err(),
                    ContractError::TokenAlreadyDeployed
                );

                let token_metadata = TokenMetadata {
                    name,
                    symbol,
                    decimal: decimals as u32,
                };

                ensure!(
                    validate_token_metadata(&token_metadata).is_ok(),
                    ContractError::InvalidTokenMetaData
                );

                // Note: attempt to convert a byte string which doesn't represent a valid Soroban address fails at the Host level
                let minter = minter
                    .map(|m| Address::from_xdr(env, &m))
                    .transpose()
                    .map_err(|_| ContractError::InvalidMinter)?;

                let deployed_address = Self::deploy_interchain_token_contract(
                    env,
                    minter,
                    token_id.clone(),
                    token_metadata,
                );

                Self::set_token_id_config(
                    env,
                    token_id,
                    TokenIdConfigValue {
                        token_address: deployed_address,
                        token_manager_type: TokenManagerType::NativeInterchainToken,
                    },
                );
            }
        };

        extend_persistent_ttl(env, &DataKey::TrustedChain(source_chain));
        extend_instance_ttl(env);

        Ok(())
    }

    fn get_execute_params(
        env: &Env,
        source_chain: String,
        payload: &Bytes,
    ) -> Result<(String, Message), ContractError> {
        let message_type = get_message_type(&payload.to_alloc_vec())?;

        ensure!(
            message_type == EncodedMessageType::ReceiveFromHub,
            ContractError::InvalidMessageType
        );

        ensure!(
            source_chain == Self::its_hub_chain_name(env),
            ContractError::InvalidHubChain
        );

        let decoded_message = HubMessage::abi_decode(env, payload)?;

        let HubMessage::ReceiveFromHub {
            source_chain: original_source_chain,
            message: inner_message,
        } = decoded_message
        else {
            return Err(ContractError::InvalidMessageType);
        };

        ensure!(
            Self::is_trusted_chain(env, original_source_chain.clone()),
            ContractError::UntrustedChain
        );

        Ok((original_source_chain, inner_message))
    }

    fn set_token_id_config(env: &Env, token_id: BytesN<32>, token_data: TokenIdConfigValue) {
        env.storage()
            .persistent()
            .set(&DataKey::TokenIdConfigKey(token_id), &token_data);
    }

    /// Retrieves the configuration value for the specified token ID.
    ///
    /// # Arguments
    /// - `env`: Reference to the environment.
    /// - `token_id`: A 32-byte unique identifier for the token.
    ///
    /// # Returns
    /// - `Ok(TokenIdConfigValue)`: The configuration value if it exists.
    /// - `Err(ContractError::InvalidTokenId)`: If the token ID does not exist in storage.
    fn token_id_config(
        env: &Env,
        token_id: BytesN<32>,
    ) -> Result<TokenIdConfigValue, ContractError> {
        env.storage()
            .persistent()
            .get::<_, TokenIdConfigValue>(&DataKey::TokenIdConfigKey(token_id))
            .ok_or(ContractError::InvalidTokenId)
    }

    /// Retrieves the configuration value for the specified token ID and extends its TTL.
    ///
    /// # Arguments
    /// - `env`: Reference to the environment.
    /// - `token_id`: A 32-byte unique identifier for the token.
    ///
    /// # Returns
    /// - `Ok(TokenIdConfigValue)`: The configuration value if it exists.
    /// - `Err(ContractError::InvalidTokenId)`: If the token ID does not exist in storage.
    fn token_id_config_with_extended_ttl(
        env: &Env,
        token_id: BytesN<32>,
    ) -> Result<TokenIdConfigValue, ContractError> {
        let config = Self::token_id_config(env, token_id.clone())?;
        extend_persistent_ttl(env, &DataKey::TokenIdConfigKey(token_id));
        Ok(config)
    }

    fn chain_name_hash(env: &Env) -> BytesN<32> {
        let chain_name = Self::chain_name(env);
        env.crypto().keccak256(&chain_name.to_xdr(env)).into()
    }

    /// Deploys a remote token on a specified destination chain.
    ///
    /// This function authorizes the caller, retrieves the token's metadata,
    /// validates the metadata, and emits an event indicating the start of the
    /// token deployment process. It also constructs and sends the deployment
    /// message to the remote chain.
    ///
    /// # Arguments
    /// * `env` - Reference to the environment object.
    /// * `caller` - Address of the caller initiating the deployment.
    /// * `deploy_salt` - Unique salt used for token deployment.
    /// * `destination_chain` - The name of the destination chain where the token will be deployed.
    /// * `gas_token` - The token used to pay for gas during the deployment.
    ///
    /// # Returns
    /// Returns the token ID of the deployed token on the remote chain, or an error if the deployment fails.
    ///
    /// # Errors
    /// Returns `ContractError` if the deployment fails, the token ID is invalid, or token metadata is invalid.
    fn deploy_remote_token(
        env: &Env,
        caller: Address,
        deploy_salt: BytesN<32>,
        destination_chain: String,
        gas_token: Token,
    ) -> Result<BytesN<32>, ContractError> {
        let token_id = Self::interchain_token_id(env, Address::zero(env), deploy_salt);
        let token_address = Self::token_id_config(env, token_id.clone())?.token_address;
        let token = token::Client::new(env, &token_address);
        let token_metadata = TokenMetadata {
            name: token.name(),
            decimal: token.decimals(),
            symbol: token.symbol(),
        };

        ensure!(
            validate_token_metadata(&token_metadata).is_ok(),
            ContractError::InvalidTokenMetaData
        );

        let message = Message::DeployInterchainToken(DeployInterchainToken {
            token_id: token_id.clone(),
            name: token_metadata.name.clone(),
            symbol: token_metadata.symbol.clone(),
            decimals: token_metadata.decimal as u8,
            minter: None,
        });

        InterchainTokenDeploymentStartedEvent {
            token_id: token_id.clone(),
            token_address,
            destination_chain: destination_chain.clone(),
            name: token_metadata.name,
            symbol: token_metadata.symbol,
            decimals: token_metadata.decimal,
            minter: None,
        }
        .emit(env);

        Self::pay_gas_and_call_contract(env, caller, destination_chain, message, gas_token)?;

        Ok(token_id)
    }

    fn deploy_interchain_token_contract(
        env: &Env,
        minter: Option<Address>,
        token_id: BytesN<32>,
        token_metadata: TokenMetadata,
    ) -> Address {
        let deployed_address = env
            .deployer()
            .with_address(env.current_contract_address(), token_id.clone())
            .deploy_v2(
                Self::interchain_token_wasm_hash(env),
                (
                    env.current_contract_address(),
                    minter.clone(),
                    token_id.clone(),
                    token_metadata.clone(),
                ),
            );

        InterchainTokenDeployedEvent {
            token_id,
            token_address: deployed_address.clone(),
            name: token_metadata.name,
            symbol: token_metadata.symbol,
            decimals: token_metadata.decimal,
            minter,
        }
        .emit(env);

        deployed_address
    }
}
