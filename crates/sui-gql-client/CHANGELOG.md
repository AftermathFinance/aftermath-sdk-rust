# Changelog

* The following workspace dependencies were updated
  * dependencies
    * sui-gql-schema bumped from 0.8.0 to 0.8.1
    * af-sui-types bumped from 0.7.0 to 0.7.1
    * af-move-type bumped from 0.8.0 to 0.8.1
  * build-dependencies
    * sui-gql-schema bumped from 0.8.0 to 0.8.1

## [0.14.1](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-gql-client-v0.14.0...sui-gql-client-v0.14.1) (2025-01-17)


### Features

* add filtered_full_objects gql query ([8640b5a](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/8640b5a9b5d47f79bb354d9eadb5f04632ef4298))


### Bug Fixes

* add new query to example in cargo.toml ([41f8790](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/41f8790cd2199f7b329250e70d3bcce0da2ae0fb))
* remove clippy unwrap_used ([37319ff](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/37319ffb84cdf69609106becf72e886330895e08))
* undo bump version ([594b5f7](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/594b5f793e4a768cbba82d64c904063e8bb59718))

## [0.14.0](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-gql-client-v0.13.4...sui-gql-client-v0.14.0) (2025-01-14)


### ⚠ BREAKING CHANGES

* **af-sui-types:** bump to 0.7.0
* **sui-gql-client,sui-gql-schema:** remove `unstable` feature
* **sui-gql-schema:** remove UInt53 scalar
* **af-sui-types:** replace TransactionEffects

### Features

* **af-sui-types:** replace TransactionEffects ([a22c555](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/a22c5558f9062c4a5111dfb1ff65ce98b9c169e1))
* **sui-gql-client,sui-gql-schema:** remove `unstable` feature ([d94a5e3](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/d94a5e3c610857f762c9e945dc1ed0cb31fd5edb))
* **sui-gql-client:** deprecate `ObjectFilter::object_keys` ([f43324f](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/f43324ff8175f8f9007672d73f39761b5ab770b4))
* **sui-gql-schema:** remove UInt53 scalar ([4c503c7](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/4c503c72bae2686951f19fbb2e24474fb69fc4b0))


### Miscellaneous Chores

* **af-sui-types:** bump to 0.7.0 ([27e110a](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/27e110a9455d4a1b9c4d9c1a9e4e0c85728a1e96))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * af-sui-types bumped from 0.7 to 0.7.0

## [0.13.4](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-gql-client-v0.13.3...sui-gql-client-v0.13.4) (2025-01-14)


### Features

* **crates:** add remaining crates ([#2](https://github.com/AftermathFinance/aftermath-sdk-rust/issues/2)) ([5d2dae1](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/5d2dae1392de8ed6a5af63a0e559bd3416112b35))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * sui-gql-schema bumped from 0.7.2 to 0.7.3
    * af-move-type bumped from 0.7.2 to 0.7.3
  * build-dependencies
    * sui-gql-schema bumped from 0.7.2 to 0.7.3

## [0.13.3](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-gql-client-v0.13.2...sui-gql-client-v0.13.3) (2025-01-14)


### Features

* **crates:** add remaining crates ([#2](https://github.com/AftermathFinance/aftermath-sdk-rust/issues/2)) ([5d2dae1](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/5d2dae1392de8ed6a5af63a0e559bd3416112b35))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * sui-gql-schema bumped from 0.7 to 0.7.2
    * af-move-type bumped from 0.7 to 0.7.2
  * build-dependencies
    * sui-gql-schema bumped from 0.7 to 0.7.2
