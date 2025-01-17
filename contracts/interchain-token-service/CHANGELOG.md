# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

### ⛰️ Features

- *(ITS)* Add chain name and token deploy salt and token sdk to ITS ([#95](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/95)) - ([017d421](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/017d421eb8c131a84de1b49fca89a45b094e2da9))
- *(ITS)* Add ITS message ABI encoding/decoding support ([#65](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/65)) - ([9c49a73](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9c49a73346d161b8bd52060ef764db04464ceb80))
- *(ITS)* Allow owner to add/remove trusted addresses ([#39](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/39)) - ([6dddb12](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/6dddb124a73c40de17b0e88aa570edeb6db4efc5))
- *(axelar-soroban-std)* Allow typed matching of emitted events ([#92](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/92)) - ([7a410ca](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7a410cab94a280777361e75c73675431b2c1be2f))
- *(interchain-token-service)* Add flow limit ([#130](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/130)) - ([7da3677](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7da36774cfa4f44d7bd66de9ac04f8ed0d4c6160))
- *(interchain-token-service)* Library export ([#126](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/126)) - ([7adc13d](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7adc13d91bd322c9d62cebcca11aa63c7d9c5cbf))
- *(interchain-token-service)* Deploy remote canonical token ([#123](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/123)) - ([bec2a07](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bec2a0723a4e42a6c1db0c435cc65f5a07898326))
- *(interchain-token-service)* Process deploy interchain token message ([#122](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/122)) - ([bfc7ded](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bfc7ded743699b4e6d50d721876cd5c2f7db293b))
- *(interchain-token-service)* Remote interchain token ([#118](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/118)) - ([6ec2622](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/6ec26221bd6a7583b65bde93a2f69a7abb4dacb9))
- *(interchain-token-service)* Register canonical interchain token ([#119](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/119)) - ([7cce18f](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7cce18fbf483648456c11c799c334430d6475c46))
- *(interchain-token-service)* Add interchain token executable ([#120](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/120)) - ([daf70ec](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/daf70ec484eb6c50fcf30ac0e0ca874fdea729ff))
- *(interchain-token-service)* Add interchain_transfer implementation ([#115](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/115)) - ([ff1f206](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/ff1f2068702f09babb3d0b3afe4a5ebee7f7bbdf))
- *(interchain-token-service)* Store token id as token data ([#112](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/112)) - ([1ddfef5](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1ddfef51c8b9fc7689e90b233b2f3d9754d8d942))
- *(interchain-token-service)* Deploy interchain token ([#99](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/99)) - ([bdf9443](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bdf9443d55f142a333a5d39a059c9f7479327ce4))
- *(interchain-token-service)* Add message routing ([#90](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/90)) - ([e06032c](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e06032cb8b90e2e8c78cd0a93a4dad4da588df75))
- *(interchain-token-service,interchain-token)* Add contract constructor ([#74](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/74)) - ([4cbaab3](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4cbaab3f1fed2878a1ad5259c40d361b85a4747f))
- *(its)* Update event test with new event test utils ([#114](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/114)) - ([1761b73](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1761b733706f3e522d7cf060b04b697da878bee7))
- *(its)* Add skeleton code for its token deployment ([#88](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/88)) - ([b062cf1](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/b062cf1eb9f26ef2ceeebeded732fd40e58f48f4))
- *(its)* Add ITS message types ([#55](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/55)) - ([9b62384](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9b6238401ccdd087c8a8d9a6516fc2537dcac8ba))
- The execute function of the ExecutableInterface trait returns a result ([#132](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/132)) - ([47d92ee](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/47d92eec27cf9dc5d7a850a1e4a70f810a75da06))
- Add extend ttl to all contracts ([#124](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/124)) - ([ab4361c](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/ab4361c58daffebd099ab386910b55a4d56d152f))
- Add macros for shared interfaces ([#105](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/105)) - ([4f513f9](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4f513f933d290cc9cc5944e5e39bcda13a136906))
- Add upgrade capabilities to all contracts ([#87](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/87)) - ([9785e8b](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9785e8bebea93e987af664cedea3234241675d96))
- Raise clippy lint level to `clippy::nursery` and apply lints ([#47](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/47)) - ([52951e1](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/52951e11f500b83f6cb31a3cadb845c4841af6a4))

### 🐛 Bug Fixes

- *(interchain-token-service)* Add destination validation checks ([#150](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/150)) - ([1b77756](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1b77756e62f35f7ac016dab7fbba9e2a06375e87))
- *(interchain-token-service)* Update error handling for token id config ([#121](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/121)) - ([8430b93](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8430b939fe3a330161ee7318218a85ee6721254d))
- *(interchain-token-service)* Revert on execute error ([#116](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/116)) - ([91533fc](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/91533fc4ff61aa3d7a3fc005cc52d8efbaf1f2ad))
- *(its)* Move ITS test directory to the correct level ([#86](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/86)) - ([6639369](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/6639369c916e2b839faa2227fc783e49704cb926))

### 🚜 Refactor

- *(interchain-token-service)* Use IntoEvent derive macro ([#143](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/143)) - ([5800cf7](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/5800cf7bed4a7efefc7bebd037635968eff3ee99))
- *(interchain-token-service)* Change trusted address to trusted chain ([#110](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/110)) - ([8798898](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8798898467d5c4e2d3e0de071386b3ca6944a53a))
- *(its)* Add BytesExt trait ([#108](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/108)) - ([0b2d38a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/0b2d38ab5cc8895e6038113db2e1f391e555b4fd))
- Update mock auth macro to support non root auth  ([#134](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/134)) - ([7b6a553](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7b6a55385fc0bdcbd7d6bf065ddaa0f81dceb51f))
- Rename assert_auth macros ([#138](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/138)) - ([8239e41](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8239e4126cdccb4156f737dd6e20fad5c2bfc239))
- [**breaking**] Update package name and references for release ([#145](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/145)) - ([bb19538](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bb195386eeda9c75d4da33eb0cf29fd9cb9b621c))
- Restrict exports to contract and contract clients ([#103](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/103)) - ([4c25023](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4c250237afce95fcd687f74e350b6b272a3d295d))
- Extract ownership management into sharable interface ([#97](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/97)) - ([df2d7d8](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/df2d7d8106e26c143757d26dfc321ffd5778d23b))
- Move shared interfaces in preparation of ownership trait extraction ([#96](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/96)) - ([e63006a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e63006a4f17abccbd1922389f1c03cc1735220b3))

### 📚 Documentation

- *(interchain-token-service)* Move entrypoint docstring to interface ([#146](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/146)) - ([f9ec5a6](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/f9ec5a66e8eb351ddea77fa23a48f57c7888adcf))

### 🧪 Testing

- Add expected invoke auth error for unit tests ([#52](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/52)) - ([890bfcf](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/890bfcfc92badf0ffed2c90aa581efdac4ce81dc))

### ⚙️ Miscellaneous Tasks

- *(its)* Move test.rs to tests folder ([#76](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/76)) - ([193c6ba](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/193c6bab049b90d80f80bc17de4d75ddd2d517bb))
- Use rustfmt nightly build to introduce opinionated imports ordering ([#141](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/141)) - ([e19f588](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e19f5887dcb7f648d1aacb0fedbd6dfa9bf45eb2))
- Simplify auth test utils ([#125](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/125)) - ([8eba319](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8eba3199c09180f7db446ddcc25580ad935fbfcc))
- Switch rust actions in workflows ([#107](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/107)) - ([480cb04](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/480cb04f311786545669a6f39a8a3a55950245e7))
- Add the support for release pipeline ([#54](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/54)) - ([90d4368](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/90d436811258b54ee8efbac074da515e977eb47e))
- Rename dev_dependencies to dev-dependencies ([#61](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/61)) - ([47c6576](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/47c657656cf83105c46b64b98d85c0653212d528))
- Remove `axelar-soroban-interfaces` crate ([#46](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/46)) - ([514d8a4](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/514d8a441ab30587dd953004894596147298fec7))

### Contributors

* @ahramy
* @nbayindirli
* @AttissNgo
* @TanvirDeol
* @cgorenflo
* @milapsheth
* @hydrobeam
* @apolikamixitos
