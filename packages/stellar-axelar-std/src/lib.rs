#![no_std]

// required by goldie
#[cfg(any(test, feature = "testutils"))]
extern crate std;

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

pub mod traits;

pub mod types;

pub mod error;

pub mod ttl;

pub mod events;

#[cfg(feature = "derive")]
pub mod interfaces;

pub mod address;

#[cfg(feature = "derive")]
pub use stellar_axelar_std_derive::*;
