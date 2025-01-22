# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

### ⛰️ Features

- *(interchain-token-service)* Add associated error types for ITS executable interface ([#142](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/142)) - ([7615a1f](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7615a1f0c73f739dc8b8a631674bfcc00c14505a))
- *(interchain-token-service)* Encode source/recipient addresses as strings instead of XDR ([#147](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/147)) - ([2b3ca63](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/2b3ca63d75535ad3260e50d72d24a07fa3cb761d))
- *(interchain-token-service)* Handle stellar native asset metadata ([#155](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/155)) - ([87cb759](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/87cb759cf9a2790e054b88b2b30fd6f03af65574))
- Simplify event definition via IntoEvent derive macro ([#136](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/136)) - ([9052c78](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9052c7886b8d2ea12f33a1fdcceaa7d159890c4e))

### 🚜 Refactor

- *(interchain-token-service)* Cleanup its execute handlers ([#157](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/157)) - ([1a5876d](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1a5876d89ac9eff147c728fd2ce778fdc2f1565c))
- Use IntoEvent derive macro for all events ([#165](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/165)) - ([2eee184](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/2eee18410d4f96fd62124bbb6eff43224c79e56d))
- Remove stellar axelar std ([#164](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/164)) - ([294b1d8](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/294b1d832002732a76bc69d8dd89174eb3c572f8))
- Update mock auth macro to support non root auth  ([#134](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/134)) - ([7b6a553](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7b6a55385fc0bdcbd7d6bf065ddaa0f81dceb51f))
- Rename assert_auth macros ([#138](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/138)) - ([8239e41](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8239e4126cdccb4156f737dd6e20fad5c2bfc239))
- [**breaking**] Update package name and references for release ([#145](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/145)) - ([bb19538](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bb195386eeda9c75d4da33eb0cf29fd9cb9b621c))

### 🧪 Testing

- Check auth is used in assert_auth ([#151](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/151)) - ([4d8e920](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4d8e92065d528cd48a08319449b80f32322e5b08))

### ⚙️ Miscellaneous Tasks

- Revert duplicated release v0.1.0 ([#168](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/168)) - ([b672e2f](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/b672e2f7515d55833c997b94667d21d1d108fd69))
- Update workspace dependencies ([#158](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/158)) - ([f214826](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/f214826c4695fdf0d25e6298a94c415fa8ea1ff0))

### Contributors

* @ahramy
* @nbayindirli
* @talalashraf
* @cgorenflo
* @milapsheth
* @TanvirDeol

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
