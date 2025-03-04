# Axelar Cross-chain Gateway Protocol for Soroban

This repo implements Axelar's [cross-chain gateway protocol](https://github.com/axelarnetwork/cgp-spec/tree/main/solidity) in Soroban for use on Stellar. The reference Solidity contracts can be found [here](https://github.com/axelarnetwork/cgp-spec/tree/main/solidity#design).

> Check configuration for CLI and Identity before deployment: https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup

## Docs

Rustdocs for this workspace can be found [here](https://axelarnetwork.github.io/axelar-cgp-stellar/).

## Install

Install Soroban CLI

```bash
cargo install --locked stellar-cli --features opt
```

## Build

```bash
cargo build
```

## Build wasm

```bash
cargo wasm

# OR

stellar contract build
```

## Test

```bash
cargo test
```

## Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov
cargo llvm-cov --html # Generate coverage report
cargo llvm-cov --open # Generate coverage and open report
```

## Optimize contracts

```bash
./optimize.sh
```

## Deployment

Deployment scripts can be found in this [repo](https://github.com/axelarnetwork/axelar-contract-deployments).
