#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

mod contract;
pub mod event;
mod storage_types;

pub use contract::{Example, ExampleClient};

#[cfg(test)]
mod tests;
