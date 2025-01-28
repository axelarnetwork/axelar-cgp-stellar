use soroban_sdk::BytesN;
use stellar_axelar_std::address::AddressExt;

use super::utils::setup_env;

// NOTE: This MUST NOT change after the initial deployment to avoid breaking existing logic
#[test]
fn interchain_token_address_is_unchanged() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::<32>::from_array(&env, &[1; 32]);

    goldie::assert!(hex::encode(
        client.interchain_token_address(&token_id).to_raw_bytes()
    ));
}

// NOTE: This MUST NOT change after the initial deployment to avoid breaking existing logic
#[test]
fn token_manager_derivation_is_unchanged() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::<32>::from_array(&env, &[1; 32]);

    goldie::assert!(hex::encode(
        client.token_manager_address(&token_id).to_raw_bytes()
    ));
}
