#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

mod contract;
pub mod event;
pub mod interface;
mod storage;

pub use contract::{AxelarExample, AxelarExampleClient};

#[cfg(test)]
mod tests;
