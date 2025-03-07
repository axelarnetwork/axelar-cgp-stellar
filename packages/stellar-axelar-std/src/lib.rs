#![no_std]

// required by goldie
#[cfg(any(test, feature = "testutils"))]
extern crate std;

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(test)]
mod tests;

pub mod traits;

pub mod string;

pub mod types;

pub mod error;

pub mod ttl;

pub mod events;

#[cfg(any(test, feature = "derive"))]
pub mod interfaces;

pub mod address;

#[cfg(any(test, feature = "derive"))]
pub use stellar_axelar_std_derive::*;
