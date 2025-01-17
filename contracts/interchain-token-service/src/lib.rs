#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod error;
pub mod executable;
mod interface;
pub mod types;

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
