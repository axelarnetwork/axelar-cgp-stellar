# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.3.0...stellar-axelar-std-v1.0.0)

### ⚙️ Miscellaneous Tasks

- Update package descriptions ([#226](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/226)) - ([1881ec7](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1881ec723644734f0c19c32db143e7a539f74ad3))

### Contributors

* @ahramy

## [0.3.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.2.2...stellar-axelar-std-v0.3.0)

### ⛰️ Features

- *(axelar-gateway)* Add more queries ([#207](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/207)) - ([ca3b486](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/ca3b4861a1a26b63cad5f12daa86a71a29107cee))
- *(axelar-std)* [**breaking**] Add pausable interface ([#204](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/204)) - ([0d4af95](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/0d4af958562e502df15dcd6bc50ec4ec66cbae46))
- *(axelar-std-derive)* Add macro to execute when contract is not paused ([#214](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/214)) - ([03d1a48](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/03d1a48b8ad9d0f4f87fc18d1ffbe6405c814fb5))
- *(token-manager)* Add token manager for ITS ([#215](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/215)) - ([42d7b34](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/42d7b348a4b419ce77c35688f93ba803c2e5ef1e))

### 🐛 Bug Fixes

- *(axelar-std-derive)* Cleanup dependencies ([#213](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/213)) - ([c986ce8](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/c986ce8f689d118e78f6d1435bbe7bffd42ad3fd))

### 🚜 Refactor

- *(interchain-token-service)* Separate ITS logic into modules ([#219](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/219)) - ([86c7bac](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/86c7bac9cf2e52d515c841dc6c4e571e12645e90))

### Contributors

* @milapsheth

## [0.2.2](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.2.1...stellar-axelar-std-v0.2.2)

### 🚜 Refactor

- Move test modules into lib.rs ([#199](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/199)) - ([51a638a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/51a638a52bdaebc4928aab9e191b28a90e73f338))

### Contributors

* @AttissNgo

## [0.2.1](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.2.0...stellar-axelar-std-v0.2.1)

### ⚙️ Miscellaneous Tasks

- Update description for packages ([#196](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/196)) - ([a20b6ab](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/a20b6ab2633b3ca407c440b9ce35ff0071384638))

### Contributors

* @ahramy

## [0.2.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.1.0...stellar-axelar-std-v0.2.0)

### 🚜 Refactor

- [**breaking**] Rename packages and move tests under src ([#185](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/185)) - ([804c962](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/804c962a667a7889c447decf8155c4f56c7b1bdb))

### Contributors

* @ahramy

## [0.1.0]

### ⛰️ Features

- Simplify event definition via IntoEvent derive macro ([#136](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/136)) - ([9052c78](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9052c7886b8d2ea12f33a1fdcceaa7d159890c4e))

### 🚜 Refactor

- Update mock auth macro to support non root auth  ([#134](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/134)) - ([7b6a553](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7b6a55385fc0bdcbd7d6bf065ddaa0f81dceb51f))
- Rename assert_auth macros ([#138](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/138)) - ([8239e41](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8239e4126cdccb4156f737dd6e20fad5c2bfc239))
- [**breaking**] Update package name and references for release ([#145](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/145)) - ([bb19538](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bb195386eeda9c75d4da33eb0cf29fd9cb9b621c))

### 🧪 Testing

- Check auth is used in assert_auth ([#151](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/151)) - ([4d8e920](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4d8e92065d528cd48a08319449b80f32322e5b08))

### Contributors

* @milapsheth
* @ahramy
* @nbayindirli
* @TanvirDeol
