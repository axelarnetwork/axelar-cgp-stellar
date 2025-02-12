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
mod tests;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{InterchainTokenServiceClient, InterchainTokenServiceInterface};
    } else {
        mod abi;
        mod deployer;
        pub mod event;
        mod storage;
        mod token_id;
        mod token_manager;
        mod token_metadata;
        mod token_handler;
        mod contract;
        mod flow_limit;

        pub use contract::{InterchainTokenService, InterchainTokenServiceClient};
    }
}
