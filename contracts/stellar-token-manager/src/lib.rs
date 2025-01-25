#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod error;

mod interface;

#[cfg(test)]
mod tests;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{TokenManagerClient, TokenManagerInterface};
    } else {
        mod contract;

        pub use contract::{TokenManager, TokenManagerClient};
    }
}
