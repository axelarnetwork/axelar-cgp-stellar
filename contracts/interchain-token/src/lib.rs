#![no_std]

pub mod error;

mod interface;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{InterchainTokenClient, InterchainTokenInterface};
    } else {
        mod event;
        mod storage_types;
        mod contract;

        pub use contract::{InterchainToken, InterchainTokenClient};
    }
}
