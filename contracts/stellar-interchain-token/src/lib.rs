#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod error;

mod interface;

#[cfg(test)]
mod tests {
    mod test;
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{InterchainTokenClient, InterchainTokenInterface};
    } else {
        pub mod event;
        mod storage_types;
        mod contract;

        pub use contract::{InterchainToken, InterchainTokenClient};
    }
}
