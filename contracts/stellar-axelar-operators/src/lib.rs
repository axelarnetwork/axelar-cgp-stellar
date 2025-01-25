#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod error;

mod interface;

pub use contract::{AxelarOperators, AxelarOperatorsClient};

#[cfg(test)]
mod tests;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{AxelarOperatorsClient, AxelarOperatorsInterface};
    } else {
        pub mod event;
        mod storage_types;
        mod contract;

        pub use contract::{AxelarOperators, AxelarOperatorsClient};
    }
}

