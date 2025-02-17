#![no_std]

#[cfg(test)]
extern crate std;

pub mod error;

mod interface;

#[cfg(test)]
mod tests;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(test)))] {
        pub use interface::{AxelarOperatorsClient, AxelarOperatorsInterface};
    } else {
        pub mod event;
        mod storage;
        mod contract;
        mod migrate;

        pub use contract::{AxelarOperators, AxelarOperatorsClient};
        // TODO: Exported to avoid dead_code warnings
        pub use interface::AxelarOperatorsInterface;
    }
}
