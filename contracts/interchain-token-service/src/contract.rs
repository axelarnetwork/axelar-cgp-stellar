use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gas_service::AxelarGasServiceClient;
use stellar_axelar_gateway::executable::AxelarExecutableInterface;
use stellar_axelar_gateway::AxelarGatewayMessagingClient;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::events::Event;
use stellar_axelar_std::ttl::{extend_instance_ttl, extend_persistent_ttl};
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{ensure, interfaces, Operatable, Ownable, Upgradable};
use stellar_interchain_token::InterchainTokenClient;

use crate::error::ContractError;
use crate::event::{
    InterchainTokenDeployedEvent, InterchainTokenDeploymentStartedEvent,
    InterchainTokenIdClaimedEvent, InterchainTransferReceivedEvent, InterchainTransferSentEvent,
    TrustedChainRemovedEvent, TrustedChainSetEvent,
};
use crate::executable::InterchainTokenExecutableClient;
use crate::flow_limit::FlowDirection;
use crate::interface::InterchainTokenServiceInterface;
use crate::storage_types::{DataKey, TokenIdConfigValue};
use crate::token_metadata::TokenMetadataExt;
use crate::types::{
    DeployInterchainToken, HubMessage, InterchainTransfer, Message, TokenManagerType,
};
use crate::{flow_limit, token_handler, token_metadata};

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
        native_token_address: Address,
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
        env.storage()
            .instance()
            .set(&DataKey::NativeTokenAddress, &native_token_address);
        env.storage().instance().set(
            &DataKey::InterchainTokenWasmHash,
            &interchain_token_wasm_hash,
        );
    }
}

#[contractimpl]
impl InterchainTokenServiceInterface for InterchainTokenService {
    fn gas_service(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::GasService)
            .expect("gas service not found")
    }

    fn chain_name(env: &Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::ChainName)
            .expect("chain name not found")
    }

    fn its_hub_chain_name(env: &Env) -> String {
        String::from_str(env, ITS_HUB_CHAIN_NAME)
    }

    fn its_hub_address(env: &Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::ItsHubAddress)
            .expect("its hub address not found")
    }

    fn native_token_address(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::NativeTokenAddress)
            .expect("native token address not found")
    }

    fn interchain_token_wasm_hash(env: &Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::InterchainTokenWasmHash)
            .expect("interchain token wasm hash not found")
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

    fn canonical_token_deploy_salt(env: &Env, token_address: Address) -> BytesN<32> {
        let chain_name_hash = Self::chain_name_hash(env);
        env.crypto()
            .keccak256(&(PREFIX_CANONICAL_TOKEN_SALT, chain_name_hash, token_address).to_xdr(env))
            .into()
    }

    fn token_address(env: &Env, token_id: BytesN<32>) -> Address {
        Self::token_id_config(env, token_id)
            .expect("token id config not found")
            .token_address
    }

    fn token_manager_type(env: &Env, token_id: BytesN<32>) -> TokenManagerType {
        Self::token_id_config(env, token_id)
            .expect("token id config not found")
            .token_manager_type
    }

    fn flow_limit(env: &Env, token_id: BytesN<32>) -> Option<i128> {
        flow_limit::flow_limit(env, token_id)
    }

    fn flow_out_amount(env: &Env, token_id: BytesN<32>) -> i128 {
        flow_limit::flow_out_amount(env, token_id)
    }

    fn flow_in_amount(env: &Env, token_id: BytesN<32>) -> i128 {
        flow_limit::flow_in_amount(env, token_id)
    }

    fn set_flow_limit(
        env: &Env,
        token_id: BytesN<32>,
        flow_limit: Option<i128>,
    ) -> Result<(), ContractError> {
        Self::operator(env).require_auth();

        flow_limit::set_flow_limit(env, token_id, flow_limit)
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

        token_metadata.validate()?;

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

        ensure!(
            !destination_address.is_empty(),
            ContractError::InvalidDestinationAddress
        );

        if let Some(ref data) = data {
            ensure!(!data.is_empty(), ContractError::InvalidData);
        }

        caller.require_auth();

        token_handler::take_token(
            env,
            &caller,
            Self::token_id_config_with_extended_ttl(env, token_id.clone())?,
            amount,
        )?;

        FlowDirection::Out.add_flow(env, token_id.clone(), amount)?;

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
            source_address: caller.to_string_bytes(),
            destination_address,
            amount,
            data,
        });

        Self::pay_gas_and_call_contract(env, caller, destination_chain, message, gas_token)?;

        Ok(())
    }
}

#[contractimpl]
impl AxelarExecutableInterface for InterchainTokenService {
    type Error = ContractError;

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
    ) -> Result<(), ContractError> {
        Self::validate_message(&env, &source_chain, &message_id, &source_address, &payload)?;

        Self::execute_message(&env, source_chain, message_id, source_address, payload)
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
        source_address: String,
        payload: Bytes,
    ) -> Result<(), ContractError> {
        let (source_chain, message) =
            Self::get_execute_params(env, source_chain, source_address, payload)?;

        match message {
            Message::InterchainTransfer(message) => {
                Self::execute_transfer_message(env, &source_chain, message_id, message)
            }
            Message::DeployInterchainToken(message) => Self::execute_deploy_message(env, message),
        }?;

        extend_persistent_ttl(env, &DataKey::TrustedChain(source_chain));
        extend_instance_ttl(env);

        Ok(())
    }

    /// Validate that the message is coming from the ITS Hub and decode the message
    fn get_execute_params(
        env: &Env,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    ) -> Result<(String, Message), ContractError> {
        ensure!(
            source_chain == Self::its_hub_chain_name(env),
            ContractError::NotHubChain
        );
        ensure!(
            source_address == Self::its_hub_address(env),
            ContractError::NotHubAddress
        );

        let HubMessage::ReceiveFromHub {
            source_chain: original_source_chain,
            message,
        } = HubMessage::abi_decode(env, &payload)?
        else {
            return Err(ContractError::InvalidMessageType);
        };
        ensure!(
            Self::is_trusted_chain(env, original_source_chain.clone()),
            ContractError::UntrustedChain
        );

        Ok((original_source_chain, message))
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
        ensure!(
            destination_chain != Self::chain_name(env),
            ContractError::InvalidDestinationChain
        );

        let token_id = Self::interchain_token_id(env, Address::zero(env), deploy_salt);
        let token_address = Self::token_id_config(env, token_id.clone())?.token_address;
        let TokenMetadata {
            name,
            symbol,
            decimal,
        } = token_metadata::token_metadata(env, &token_address, &Self::native_token_address(env))?;

        let message = Message::DeployInterchainToken(DeployInterchainToken {
            token_id: token_id.clone(),
            name: name.clone(),
            symbol: symbol.clone(),
            decimals: decimal as u8,
            minter: None,
        });

        InterchainTokenDeploymentStartedEvent {
            token_id: token_id.clone(),
            token_address,
            destination_chain: destination_chain.clone(),
            name,
            symbol,
            decimals: decimal,
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

    fn execute_transfer_message(
        env: &Env,
        source_chain: &String,
        message_id: String,
        InterchainTransfer {
            token_id,
            source_address,
            destination_address,
            amount,
            data,
        }: InterchainTransfer,
    ) -> Result<(), ContractError> {
        let destination_address = Address::from_string_bytes(&destination_address);

        let token_config_value = Self::token_id_config_with_extended_ttl(env, token_id.clone())?;

        FlowDirection::In.add_flow(env, token_id.clone(), amount)?;

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
            let executable = InterchainTokenExecutableClient::new(env, &destination_address);
            executable.execute_with_interchain_token(
                source_chain,
                &message_id,
                &source_address,
                &payload,
                &token_id,
                &token_address,
                &amount,
            );
        }

        Ok(())
    }

    fn execute_deploy_message(
        env: &Env,
        DeployInterchainToken {
            token_id,
            name,
            symbol,
            decimals,
            minter,
        }: DeployInterchainToken,
    ) -> Result<(), ContractError> {
        ensure!(
            Self::token_id_config(env, token_id.clone()).is_err(),
            ContractError::TokenAlreadyDeployed
        );

        let token_metadata = TokenMetadata::new(name, symbol, decimals as u32)?;

        // Note: attempt to convert a byte string which doesn't represent a valid Soroban address fails at the Host level
        let minter = minter.map(|m| Address::from_string_bytes(&m));

        let deployed_address =
            Self::deploy_interchain_token_contract(env, minter, token_id.clone(), token_metadata);

        Self::set_token_id_config(
            env,
            token_id,
            TokenIdConfigValue {
                token_address: deployed_address,
                token_manager_type: TokenManagerType::NativeInterchainToken,
            },
        );

        Ok(())
    }
}
