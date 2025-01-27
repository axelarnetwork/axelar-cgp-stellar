#![no_std]

#[cfg(test)]
extern crate std;

pub mod error;

mod interface;

#[cfg(test)]
mod tests;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(test)))] {
        pub use interface::{TokenManagerClient, TokenManagerInterface};
    } else {
        mod contract;

        pub use contract::{TokenManager, TokenManagerClient};
    }
}
