#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod error;

mod interface;

#[cfg(all(target_family = "wasm", feature = "testutils"))]
compile_error!("'testutils' feature is not supported on 'wasm' target");

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(test)]
mod tests {
    mod test;
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{AxelarGasServiceClient, AxelarGasServiceInterface};
    } else {
        pub mod event;
        mod storage_types;
        mod contract;

        pub use contract::{AxelarGasService, AxelarGasServiceClient};
    }
}
