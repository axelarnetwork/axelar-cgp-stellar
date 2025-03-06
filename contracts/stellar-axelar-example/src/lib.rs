#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

mod contract;
pub mod event;
pub mod interface;
mod storage;

pub use contract::{Example, ExampleClient};

#[cfg(test)]
mod tests;
