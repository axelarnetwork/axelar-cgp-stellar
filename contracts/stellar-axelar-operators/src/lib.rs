#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod error;

mod interface;

#[cfg(test)]
mod tests;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{AxelarOperatorsClient, AxelarOperatorsInterface};
    } else {
        pub mod event;
        mod storage;
        mod contract;
        mod migrate;

        pub use contract::{AxelarOperators, AxelarOperatorsClient};
    }
}
