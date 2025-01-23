#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod error;
pub mod executable;
mod interface;
pub mod types;

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(test)]
mod tests {
    mod deploy_interchain_token;
    mod deploy_remote_canonical_token;
    mod deploy_remote_interchain_token;
    mod executable;
    mod execute;
    mod flow_limit;
    mod interchain_transfer;
    mod message_routing;
    mod pause;
    mod register_canonical_token;
    mod trusted_chain;
    mod utils;
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{InterchainTokenServiceClient, InterchainTokenServiceInterface};
    } else {
        mod abi;
        pub mod event;
        mod storage_types;
        mod token_metadata;
        mod token_handler;
        mod contract;
        mod flow_limit;

        pub use contract::{InterchainTokenService, InterchainTokenServiceClient};
    }
}
