use soroban_sdk::{Env, String, Vec};
use stellar_axelar_std::ensure;
use stellar_axelar_std::interfaces::CustomMigratableInterface;

use crate::contract::AxelarGateway;
use crate::error::ContractError;
use crate::storage;

pub mod legacy_storage {
    use soroban_sdk::{contracttype, BytesN, String};
    use stellar_axelar_std::contractstorage;

    use crate::storage::MessageApprovalValue;

    #[contracttype]
    #[derive(Clone, Debug)]
    pub struct MessageApprovalKey {
        pub source_chain: String,
        pub message_id: String,
    }

    #[contractstorage]
    enum LegacyDataKey {
        #[persistent]
        #[value(MessageApprovalValue)]
        MessageApproval {
            message_approval_key: MessageApprovalKey,
        },

        #[instance]
        #[value(u64)]
        PreviousSignerRetention,

        #[instance]
        #[value(BytesN<32>)]
        DomainSeparator,

        #[instance]
        #[value(u64)]
        MinimumRotationDelay,

        #[instance]
        #[value(u64)]
        Epoch,

        #[instance]
        #[value(u64)]
        LastRotationTimestamp,

        #[persistent]
        #[value(BytesN<32>)]
        SignersHashByEpoch { epoch: u64 },

        #[persistent]
        #[value(u64)]
        EpochBySignersHash { signers_hash: BytesN<32> },
    }
}

impl CustomMigratableInterface for AxelarGateway {
    type MigrationData = Vec<(String, String)>;
    type Error = ContractError;

    fn __migrate(_env: &Env, _migration_data: Self::MigrationData) -> Result<(), Self::Error> {
        for (source_chain, message_id) in _migration_data {
            let message_approval_key = legacy_storage::MessageApprovalKey {
                source_chain,
                message_id,
            };

            ensure!(
                legacy_storage::try_message_approval(_env, message_approval_key.clone()).is_some(),
                ContractError::MessageApprovalNotFound
            );

            let message_approval =
                legacy_storage::message_approval(_env, message_approval_key.clone());

            storage::set_message_approval(
                _env,
                message_approval_key.source_chain.clone(),
                message_approval_key.message_id.clone(),
                &message_approval,
            );
        }

        Ok(())
    }
}
