#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod event;
mod storage_types;

mod contract;
pub mod error;

pub use contract::{AxelarOperators, AxelarOperatorsClient};

#[cfg(test)]
mod tests;
