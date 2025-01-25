#![no_std]
#[cfg(test)]
extern crate alloc;

pub mod error;

mod interface;

pub use contract::{Upgrader, UpgraderClient};

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
