use alloy_primitives::{FixedBytes, Uint, U256};
use alloy_sol_types::{sol, SolValue};
use soroban_sdk::{Bytes, BytesN, Env, String};
use stellar_axelar_std::ensure;

// alloc needed for converting to alloy types
use crate::abi::alloc::{string::String as StdString, vec};
use crate::error::ContractError;
use crate::types::{self, HubMessage, Message};
extern crate alloc;

sol! {
    #[derive(PartialEq, Eq)]
    enum MessageType {
        InterchainTransfer,
        DeployInterchainToken,
        DeployTokenManager, // note, this case is not supported by the ITS hub
        SendToHub,
        ReceiveFromHub
    }

    struct InterchainTransfer {
        uint256 messageType;
        bytes32 tokenId;
        bytes sourceAddress;
        bytes destinationAddress;
        uint256 amount;
        bytes data;
    }

    struct DeployInterchainToken {
        uint256 messageType;
        bytes32 tokenId;
        string name;
        string symbol;
        uint8 decimals;
        bytes minter;
    }

    struct SendToHub {
        uint256 messageType;
        string destination_chain;
        bytes message;
    }

    struct ReceiveFromHub {
        uint256 messageType;
        string source_chain;
        bytes message;
    }
}

impl Message {
    pub fn abi_encode(self, env: &Env) -> Result<Bytes, ContractError> {
        let msg = match self {
            Self::InterchainTransfer(types::InterchainTransfer {
                token_id,
                source_address,
                destination_address,
                amount,
                data,
            }) => InterchainTransfer {
                messageType: MessageType::InterchainTransfer.into(),
                tokenId: FixedBytes::<32>::new(token_id.into()),
                sourceAddress: source_address.to_alloc_vec().into(),
                destinationAddress: destination_address.to_alloc_vec().into(),
                amount: amount.try_into().expect("failed to convert"),
                data: into_vec(data).into(),
            }
            .abi_encode_params(),
            Self::DeployInterchainToken(types::DeployInterchainToken {
                token_id,
                name,
                symbol,
                decimals,
                minter,
            }) => DeployInterchainToken {
                messageType: MessageType::DeployInterchainToken.into(),
                tokenId: FixedBytes::<32>::new(token_id.into()),
                name: to_std_string(name)?,
                symbol: to_std_string(symbol)?,
                decimals,
                minter: into_vec(minter).into(),
            }
            .abi_encode_params(),
        };
        Ok(Bytes::from_slice(env, &msg))
    }

    pub fn abi_decode(env: &Env, payload: &Bytes) -> Result<Self, ContractError> {
        let payload = payload.to_alloc_vec();

        let message_type = get_message_type(&payload)?;

        match message_type {
            MessageType::InterchainTransfer => {
                let decoded = InterchainTransfer::abi_decode_params(&payload, true)
                    .map_err(|_| ContractError::AbiDecodeFailed)?;

                Ok(Self::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(env, &decoded.tokenId.into()),
                    source_address: Bytes::from_slice(env, decoded.sourceAddress.as_ref()),
                    destination_address: Bytes::from_slice(
                        env,
                        decoded.destinationAddress.as_ref(),
                    ),
                    amount: to_i128(decoded.amount)?,
                    data: from_vec(env, decoded.data.as_ref()),
                }))
            }
            MessageType::DeployInterchainToken => {
                let decoded = DeployInterchainToken::abi_decode_params(&payload, true)
                    .map_err(|_| ContractError::AbiDecodeFailed)?;

                Ok(Self::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(env, &decoded.tokenId.into()),
                    name: String::from_str(env, &decoded.name),
                    symbol: String::from_str(env, &decoded.symbol),
                    decimals: decoded.decimals,
                    minter: from_vec(env, decoded.minter.as_ref()),
                }))
            }
            _ => Err(ContractError::InvalidMessageType),
        }
    }
}

impl HubMessage {
    pub fn abi_encode(self, env: &Env) -> Result<Bytes, ContractError> {
        let msg = match self {
            Self::SendToHub {
                destination_chain,
                message,
            } => SendToHub {
                messageType: MessageType::SendToHub.into(),
                destination_chain: to_std_string(destination_chain)?,
                message: message.abi_encode(env)?.to_alloc_vec().into(),
            }
            .abi_encode_params(),
            Self::ReceiveFromHub {
                source_chain,
                message,
            } => ReceiveFromHub {
                messageType: MessageType::ReceiveFromHub.into(),
                source_chain: to_std_string(source_chain)?,
                message: message.abi_encode(env)?.to_alloc_vec().into(),
            }
            .abi_encode_params(),
        };
        Ok(Bytes::from_slice(env, &msg))
    }

    pub fn abi_decode(env: &Env, payload: &Bytes) -> Result<Self, ContractError> {
        let payload = payload.to_alloc_vec();

        let message_type = get_message_type(&payload)?;

        match message_type {
            MessageType::SendToHub => {
                let decoded = SendToHub::abi_decode_params(&payload, true)
                    .map_err(|_| ContractError::AbiDecodeFailed)?;

                Ok(Self::SendToHub {
                    destination_chain: String::from_str(env, &decoded.destination_chain),
                    message: Message::abi_decode(
                        env,
                        &Bytes::from_slice(env, decoded.message.as_ref()),
                    )?,
                })
            }
            MessageType::ReceiveFromHub => {
                let decoded = ReceiveFromHub::abi_decode_params(&payload, true)
                    .map_err(|_| ContractError::AbiDecodeFailed)?;

                Ok(Self::ReceiveFromHub {
                    source_chain: String::from_str(env, &decoded.source_chain),
                    message: Message::abi_decode(
                        env,
                        &Bytes::from_slice(env, decoded.message.as_ref()),
                    )?,
                })
            }
            _ => Err(ContractError::InvalidMessageType),
        }
    }
}

pub fn get_message_type(payload: &[u8]) -> Result<MessageType, ContractError> {
    ensure!(
        payload.len() >= 32,
        ContractError::InsufficientMessageLength
    );

    let message_type = MessageType::abi_decode(&payload[0..32], true)
        .map_err(|_| ContractError::InvalidMessageType)?;

    Ok(message_type)
}

fn to_std_string(soroban_string: String) -> Result<StdString, ContractError> {
    let length = soroban_string.len() as usize;
    let mut bytes = vec![0u8; length];
    soroban_string.copy_into_slice(&mut bytes);
    StdString::from_utf8(bytes).map_err(|_| ContractError::InvalidUtf8)
}

fn to_i128(value: Uint<256, 4>) -> Result<i128, ContractError> {
    let slice = value.as_le_slice();

    let mut bytes_to_remove = [0; 16];
    let mut bytes_to_convert = [0; 16];
    bytes_to_convert.copy_from_slice(&slice[..16]);
    bytes_to_remove.copy_from_slice(&slice[16..]);

    ensure!(
        i128::from_le_bytes(bytes_to_remove) == 0,
        ContractError::InvalidAmount
    );

    let i128_value = i128::from_le_bytes(bytes_to_convert);

    ensure!(i128_value >= 0, ContractError::InvalidAmount);

    Ok(i128_value)
}

fn into_vec(value: Option<Bytes>) -> alloc::vec::Vec<u8> {
    value.map(|d| d.to_alloc_vec()).unwrap_or_default()
}

fn from_vec(env: &Env, value: &[u8]) -> Option<Bytes> {
    if value.is_empty() {
        None
    } else {
        Some(Bytes::from_slice(env, value))
    }
}

impl From<MessageType> for U256 {
    fn from(value: MessageType) -> Self {
        Self::from(value as u8)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use std::vec::Vec;

    use soroban_sdk::{Bytes, BytesN, Env, String};
    use stellar_axelar_std::assert_ok;
    use stellar_axelar_std::traits::BytesExt;

    use super::*;

    #[test]
    fn soroban_str_to_std_string() {
        let env = Env::default();

        let plain_string = "hello world";
        let plain_string_soroban = String::from_str(&env, plain_string);
        assert_eq!(
            to_std_string(plain_string_soroban).unwrap(),
            StdString::from(plain_string)
        );

        let var_length_chars = "🎉中🚀π🌈€";
        let var_length_chars_soroban = String::from_str(&env, var_length_chars);
        assert_eq!(
            to_std_string(var_length_chars_soroban).unwrap(),
            StdString::from(var_length_chars)
        );

        let null_bytes = "Hello\x00World";
        let null_bytes_soroban = String::from_str(&env, null_bytes);
        assert_eq!(
            to_std_string(null_bytes_soroban).unwrap(),
            StdString::from(null_bytes)
        );

        let escape_char = "Hello\tWorld";
        let escape_char_soroban = String::from_str(&env, escape_char);
        assert_eq!(
            to_std_string(escape_char_soroban).unwrap(),
            StdString::from(escape_char)
        );
    }

    #[test]
    fn to_std_string_fails_invalid_utf8_bytes() {
        let env = Env::default();

        let invalid_sequences = vec![
            String::from_bytes(&env, &[0xF5, 0x90, 0x80]), // not valid utf-8
            String::from_bytes(&env, &[0x00, 0x01, 0x02, 0xC0]), // valid ASCII characters followed by an invalid UTF-8 starting byte
            String::from_bytes(&env, &[0xC0, 0x80, 0xF5, 0x90]), // invalid UTF-8 starting byte followed by valid UTF-8 sequences
            String::from_bytes(&env, &[0xF0, 0x90, 0x80, 0xDF, 0xFB, 0xBF]), // surrogate pairs with invalid charaters "\uD800\uDDFF"
            String::from_bytes(&env, &[0xF4, 0x90, 0x80, 0x80]), // outside the Basic Multilingual Plane
        ];

        for sequence in invalid_sequences {
            let result = to_std_string(sequence);
            assert!(matches!(result, Err(ContractError::InvalidUtf8)));
        }
    }

    #[test]
    fn uint256_to_i128() {
        let uint_i128_max: Uint<256, 4> = i128::MAX.try_into().unwrap();

        assert_eq!(to_i128(uint_i128_max).unwrap(), i128::MAX);

        let uint_min: Uint<256, 4> = Uint::MIN;

        assert_eq!(to_i128(uint_min).unwrap(), 0);
    }

    #[test]
    fn to_i128_fails_dirty_bytes() {
        let uint_min: Uint<256, 4> = Uint::from(1);
        let bytes: [u8; 32] = uint_min.to_le_bytes();
        assert_eq!(
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ],
            bytes
        );

        let bad_bytes = [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ];
        let bad_uint = U256::from_le_bytes(bad_bytes);

        let result = to_i128(bad_uint);

        assert!(matches!(result, Err(ContractError::InvalidAmount)));
    }

    #[test]
    fn to_i128_fails_overflow() {
        let overflow: Uint<256, 4> = Uint::from(i128::MAX) + Uint::from(1);
        let result = to_i128(overflow);
        assert!(matches!(result, Err(ContractError::InvalidAmount)));

        let overflow: Uint<256, 4> = Uint::from(u128::MAX);
        let result = to_i128(overflow);
        assert!(matches!(result, Err(ContractError::InvalidAmount)));
    }

    #[test]
    fn interchain_transfer_encode_decode() {
        let env = Env::default();
        let remote_chain = String::from_str(&env, "chain");

        let cases = vec![
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    source_address: Bytes::from_hex(&env, "00"),
                    destination_address: Bytes::from_hex(&env, "00"),
                    amount: 1u64.into(),
                    data: None,
                }),
            },
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[255u8; 32]),
                    source_address: Bytes::from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    destination_address: Bytes::from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    amount: i128::MAX,
                    data: Some(Bytes::from_hex(&env, "abcd")),
                }),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    source_address: Bytes::from_hex(&env, "00"),
                    destination_address: Bytes::from_hex(&env, "00"),
                    amount: 1u64.into(),
                    data: None,
                }),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain,
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[255u8; 32]),
                    source_address: Bytes::from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    destination_address: Bytes::from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    amount: i128::MAX,
                    data: Some(Bytes::from_hex(&env, "abcd")),
                }),
            },
        ];

        let encoded: Vec<_> = cases
            .iter()
            .map(|original| {
                hex::encode(
                    assert_ok!(original.clone().abi_encode(&env))
                        .to_buffer::<1024>()
                        .as_slice(),
                )
            })
            .collect();

        goldie::assert_json!(encoded);

        for original in cases {
            let encoded = assert_ok!(original.clone().abi_encode(&env));
            let decoded = HubMessage::abi_decode(&env, &encoded);
            assert_eq!(original, decoded.unwrap());
        }
    }

    #[test]
    fn deploy_interchain_token_encode_decode() {
        let env = Env::default();
        let remote_chain = String::from_str(&env, "chain");

        let cases = vec![
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, "t"),
                    symbol: String::from_str(&env, "T"),
                    decimals: 0,
                    minter: None,
                }),
            },
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[1u8; 32]),
                    name: String::from_str(&env, "Test Token"),
                    symbol: String::from_str(&env, "TST"),
                    decimals: 18,
                    minter: Some(Bytes::from_hex(&env, "1234")),
                }),
            },
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, "Unicode Token 🪙"),
                    symbol: String::from_str(&env, "UNI🔣"),
                    decimals: 255,
                    minter: Some(Bytes::from_hex(&env, "abcd")),
                }),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, "t"),
                    symbol: String::from_str(&env, "T"),
                    decimals: 0,
                    minter: None,
                }),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[1u8; 32]),
                    name: String::from_str(&env, "Test Token"),
                    symbol: String::from_str(&env, "TST"),
                    decimals: 18,
                    minter: Some(Bytes::from_hex(&env, "1234")),
                }),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain,
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, "Unicode Token 🪙"),
                    symbol: String::from_str(&env, "UNI🔣"),
                    decimals: 255,
                    minter: Some(Bytes::from_hex(&env, "abcd")),
                }),
            },
        ];

        let encoded: Vec<_> = cases
            .iter()
            .map(|original| {
                hex::encode(
                    assert_ok!(original.clone().abi_encode(&env))
                        .to_buffer::<1024>()
                        .as_slice(),
                )
            })
            .collect();

        goldie::assert_json!(encoded);

        for original in cases {
            let encoded = assert_ok!(original.clone().abi_encode(&env));
            let decoded = HubMessage::abi_decode(&env, &encoded);
            assert_eq!(original, decoded.unwrap());
        }
    }

    #[test]
    fn abi_decode_fails_invalid_message_type() {
        let env = Env::default();
        let bytes = [0u8; 32];
        let invalid_payload = Bytes::from_slice(&env, &bytes);

        let result = HubMessage::abi_decode(&env, &invalid_payload);
        assert!(matches!(result, Err(ContractError::InvalidMessageType)));

        let invalid_hub_message_type = assert_ok!(types::Message::InterchainTransfer(
            types::InterchainTransfer {
                token_id: BytesN::from_array(&env, &[0u8; 32]),
                source_address: Bytes::from_hex(&env, "00"),
                destination_address: Bytes::from_hex(&env, "00"),
                amount: 1u64.into(),
                data: None,
            }
        )
        .abi_encode(&env));

        let result = HubMessage::abi_decode(&env, &invalid_hub_message_type);
        assert!(matches!(result, Err(ContractError::InvalidMessageType)));
    }
}
