use soroban_sdk::{Address, BytesN};

use super::utils::setup_env;

// NOTE: This MUST NOT change after the initial deployment to avoid breaking existing logic
#[test]
fn canonical_interchain_token_id_is_unchanged() {
    let (env, client, _, _, _) = setup_env();
    let token_address = Address::from_str(
        &env,
        "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
    );

    goldie::assert!(hex::encode(
        client
            .canonical_interchain_token_id(&token_address)
            .to_array()
    ));
}

// NOTE: This MUST NOT change after the initial deployment to avoid breaking existing logic
#[test]
fn interchain_token_id_is_unchanged() {
    let (env, client, _, _, _) = setup_env();
    let deployer = Address::from_str(
        &env,
        "GDUITDF2LI3R5HM4KYRLLNRLEWKYBFVZVOEB6HSL7EOW2KO2LD6V4GPM",
    );
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);

    goldie::assert!(hex::encode(
        client.interchain_token_id(&deployer, &salt).to_array()
    ));
}
