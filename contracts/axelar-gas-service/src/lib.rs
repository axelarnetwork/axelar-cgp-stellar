#![no_std]

pub mod error;

mod interface;

#[cfg(all(target_family = "wasm", feature = "testutils"))]
compile_error!("'testutils' feature is not supported on 'wasm' target");

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{AxelarGasServiceClient, AxelarGasServiceInterface};
    } else {
        mod event;
        mod storage_types;
        mod contract;

        pub use contract::{AxelarGasService, AxelarGasServiceClient};
    }
}
