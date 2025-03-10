#![no_std]

// Allows using std (and its macros) in test modules
#[cfg(any(test, feature = "testutils"))]
#[macro_use]
extern crate std;

pub mod error;
pub mod executable;
mod messaging_interface;
pub mod types;
pub use messaging_interface::{AxelarGatewayMessagingClient, AxelarGatewayMessagingInterface};

mod interface;

#[cfg(all(target_family = "wasm", feature = "testutils"))]
compile_error!("'testutils' feature is not supported on 'wasm' target");

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(test)]
mod tests;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{AxelarGatewayClient, AxelarGatewayInterface};
    } else {
        mod auth;
        pub mod event;
        mod storage;
        mod contract;
        mod migrate;

        pub use contract::{AxelarGateway, AxelarGatewayClient};
    }
}
