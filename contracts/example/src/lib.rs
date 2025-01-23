#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

mod contract;
pub mod event;
mod storage_types;
pub mod interface;

pub use contract::{Example, ExampleClient};
