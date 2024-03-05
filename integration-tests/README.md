# Integration Tests

This directory contains integration tests for Axelar contracts in Soroban.
[package]
name = "axelar-executable"
version = "0.1.0"
edition = { workspace = true }

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
soroban-sdk = { workspace = true }
axelar-gateway = { workspace = true }

[dev_dependencies]
soroban-sdk = { workspace = true, features = ["testutils"] }
axelar-soroban-std = { workspace = true, features = ["testutils"] }
axelar-auth-verifier = { workspace = true, features = [ "testutils" ] }
axelar-gateway = { workspace = true, features = [ "testutils" ] }