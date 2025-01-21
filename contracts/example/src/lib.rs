#![no_std]

#[cfg(any(test, feature = "testutils"))]
#[macro_use]
extern crate std;

mod contract;
pub mod event;
mod storage_types;

pub use contract::{Example, ExampleClient};
