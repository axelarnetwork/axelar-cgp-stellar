use soroban_sdk::{vec, Address, Env, IntoVal, Symbol, Val};
use stellar_token_manager::TokenManagerClient;

pub trait TokenManagerClientExt {
    /// Transfer `amount` of tokens from the token manager to `recipient`.
    fn transfer(&self, env: &Env, token_address: &Address, recipient: &Address, amount: i128);

    /// Mint `amount` of tokens to `recipient`.
    fn mint(&self, env: &Env, token_address: &Address, recipient: &Address, amount: i128);
}

impl TokenManagerClientExt for TokenManagerClient<'_> {
    fn transfer(&self, env: &Env, token_address: &Address, recipient: &Address, amount: i128) {
        let _: Val = self.execute(
            token_address,
            &Symbol::new(env, "transfer"),
            &vec![
                env,
                self.address.to_val(),
                recipient.to_val(),
                amount.into_val(env),
            ],
        );
    }

    fn mint(&self, env: &Env, token_address: &Address, recipient: &Address, amount: i128) {
        let _: Val = self.execute(
            token_address,
            &Symbol::new(env, "mint_from"),
            &vec![
                env,
                self.address.to_val(),
                recipient.to_val(),
                amount.into_val(env),
            ],
        );
    }
}
