#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

#[cfg(test)]
extern crate alloc;

pub mod error;

pub mod interface;

#[cfg(test)]
mod tests;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{UpgraderClient, UpgraderInterface};
    } else {
        mod contract;

        pub use contract::{Upgrader, UpgraderClient};
    }
}
