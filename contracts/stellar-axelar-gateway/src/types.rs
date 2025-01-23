use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contracttype, Address, BytesN, Env, String, Vec};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedSigner {
    pub signer: BytesN<32>, // Ed25519 public key
    pub weight: u128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedSigners {
    pub signers: Vec<WeightedSigner>,
    pub threshold: u128,
    pub nonce: BytesN<32>,
}

/// `ProofSignature` represents an optional signature from a signer.
/// Since Soroban doesn't support use of `Option` in it's contract interfaces,
/// we use this enum instead.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofSignature {
    Signed(BytesN<64>), // Ed25519 signature
    Unsigned,
}

/// `ProofSigner` represents a signer in a proof.
///
/// If the signer submitted a signature, and if it is being included in the proof to meet the threshold,
/// then a signature is attached. Otherwise, the `ProofSignature` is `Unsigned`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofSigner {
    pub signer: WeightedSigner,
    pub signature: ProofSignature,
}

/// `Proof` represents a proof that a set of signers have signed a message.
/// All weighted signers are included in the along with a signature, if they have signed the message,
/// until threshold is met.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub signers: Vec<ProofSigner>,
    pub threshold: u128,
    pub nonce: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandType {
    ApproveMessages,
    RotateSigners,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub source_chain: String,
    pub message_id: String,
    pub source_address: String,
    pub contract_address: Address,
    pub payload_hash: BytesN<32>,
}

impl WeightedSigners {
    pub fn hash(&self, env: &Env) -> BytesN<32> {
        env.crypto().keccak256(&self.clone().to_xdr(env)).into()
    }

    pub fn signers_rotation_hash(&self, env: &Env) -> BytesN<32> {
        env.crypto()
            .keccak256(&(CommandType::RotateSigners, self.clone()).to_xdr(env))
            .into()
    }
}

impl Proof {
    /// Get the weighted signers from the proof.
    pub fn weighted_signers(&self) -> WeightedSigners {
        let mut signers = Vec::new(self.signers.env());

        for ProofSigner { signer, .. } in self.signers.iter() {
            signers.push_back(signer);
        }

        WeightedSigners {
            signers,
            threshold: self.threshold,
            nonce: self.nonce.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use hex_literal::hex;
    use soroban_sdk::xdr::ToXdr;
    use soroban_sdk::{Address, BytesN, Env, String, Vec};

    use crate::types::{CommandType, Message, WeightedSigner, WeightedSigners};

    #[test]
    fn weighted_signers_hash() {
        let env = Env::default();

        let signers = [
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("0a245a2a2a5e8ec439d1377579a08fc78ea55647ba6fcb1f5d8a360218e8a985"),
                ),
                weight: 3,
            },
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("0b422cf449d900f6f8eb97f62e35811c62eb75feb84dfccef44a5c1c3dbac2ad"),
                ),
                weight: 2,
            },
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("18c34bf01a11b5ba21ea11b1678f3035ef753f0bdb1d5014ec21037e8f99e2a2"),
                ),
                weight: 4,
            },
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("f683ca8a6d7fe55f25599bb64b01edcc5eeb85fe5b63d3a4f0b3c32405005518"),
                ),
                weight: 4,
            },
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("fbb4b870e800038f1379697fae3058938c59b696f38dd0fdf2659c0cf3a5b663"),
                ),
                weight: 2,
            },
        ];

        let weighted_signers = WeightedSigners {
            signers: Vec::from_array(&env, signers),
            threshold: 8u128,
            nonce: BytesN::<32>::from_array(
                &env,
                &hex!("8784bf7be5a9baaeea47e12d9e8ad0dec29afcbc3617d97f771e3c24fa945dce"),
            ),
        };

        let hash = weighted_signers.hash(&env).to_array();
        let signers_rotation_hash = weighted_signers.signers_rotation_hash(&env).to_array();

        goldie::assert_json!(vec![hex::encode(hash), hex::encode(signers_rotation_hash)]);
    }

    #[test]
    fn messages_approval_hash() {
        let env = Env::default();

        let payload_hashes = [
            hex!("cfa347779c9b646ddf628c4da721976ceb998f1ab2c097b52e66a575c3975a6c"),
            hex!("fb5eb8245e3b8eb9d44f228ee142a3378f57d49fc95fa78d437ff8aa5dd564ba"),
            hex!("90e3761c0794fbbd8b563a0d05d83395e7f88f64f30eebb7c5533329f6653e84"),
            hex!("60e146cb9c548ba6e614a87910d8172c9d21279a3f8f4da256ff36e15b80ea30"),
        ]
        .map(|hash| BytesN::<32>::from_array(&env, &hash));

        let mut messages = soroban_sdk::Vec::new(&env);

        for (i, payload_hash) in payload_hashes.into_iter().enumerate() {
            messages.push_back(Message {
                source_chain: String::from_str(&env, &format!("source-{}", i + 1)),
                message_id: String::from_str(&env, &format!("test-{}", i + 1)),
                source_address: String::from_str(
                    &env,
                    "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHK3M",
                ),
                contract_address: Address::from_string(&String::from_str(
                    &env,
                    "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4",
                )),
                payload_hash,
            });
        }

        let approval_hash = env
            .crypto()
            .keccak256(&(CommandType::ApproveMessages, messages).to_xdr(&env))
            .to_array();

        goldie::assert!(hex::encode(approval_hash));
    }
}
