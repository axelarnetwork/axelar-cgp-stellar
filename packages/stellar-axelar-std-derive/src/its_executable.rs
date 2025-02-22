use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

pub fn its_executable(name: &Ident) -> TokenStream2 {
    quote! {
        use stellar_interchain_token_service::executable::InterchainTokenExecutableInterface as _;

        impl stellar_interchain_token_service::executable::DeriveOnly for #name {}

        #[contractimpl]
        impl stellar_interchain_token_service::executable::InterchainTokenExecutableInterface for #name {
            fn interchain_token_service(env: &Env) -> soroban_sdk::Address {
                <Self as stellar_interchain_token_service::executable::CustomInterchainTokenExecutable>::__interchain_token_service(env)
            }

            fn execute_with_interchain_token(
                env: &Env,
                source_chain: String,
                message_id: String,
                source_address: Bytes,
                payload: Bytes,
                token_id: BytesN<32>,
                token_address: Address,
                amount: i128,
            ) -> Result<(), soroban_sdk::Error> {
                    <Self as stellar_interchain_token_service::executable::CustomInterchainTokenExecutable>::__interchain_token_service(env).require_auth();
                    <Self as stellar_interchain_token_service::executable::CustomInterchainTokenExecutable>::__authorized_execute_with_token(
                        env,
                        source_chain,
                        message_id,
                        source_address,
                        payload,
                        token_id,
                        token_address,
                        amount,
                    ).map_err(|error| error.into())
            }
        }
    }
}
