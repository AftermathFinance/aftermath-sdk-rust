# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

## [0.14.3](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.14.2...sui-jsonrpc-v0.14.3)

### 🚜 Refactor

- Address clippy lints in Rust 1.86 - ([c6855e1](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/c6855e1d290ab3bdd2bba3ae9ddb24502c4ce58a))


## [0.14.2](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.14.1...sui-jsonrpc-v0.14.2)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: af-sui-types - ([0000000](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/0000000))


## [0.14.1](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.14.0...sui-jsonrpc-v0.14.1)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: af-sui-types - ([0000000](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/0000000))


## [0.13.0](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.12.8...sui-jsonrpc-v0.13.0)

### ⛰️ Features

- *(sui-jsonrpc)* [**breaking**] Add `DryRunTransactionBlockResponse::execution_error_source` - ([ed330f7](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/ed330f7612cf88715603f1a9250baf1e8994cc36))

### 🐛 Bug Fixes

- *(sui-jsonrpc)* Recreate `UserSignature` serialization used in the RPC - ([7c5f652](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/7c5f652400fade77ef3679ee8e8957b9433ad481))


## [0.12.8](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.12.7...sui-jsonrpc-v0.12.8)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: af-sui-types - ([0000000](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/0000000))


## [0.12.7](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.12.6...sui-jsonrpc-v0.12.7)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: af-sui-types - ([0000000](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/0000000))


## [0.12.5](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.12.4...sui-jsonrpc-v0.12.5)

### 📚 Documentation

- Standardize changelogs - ([383b40d](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/383b40d75c38f637aafe06438673f71e1c57d432))


## [0.12.4](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.12.3...sui-jsonrpc-v0.12.4)

### 📚 Documentation

- Regenerate changelogs from scratch - ([288008f](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/288008f5b60193ea34b765d8ad605cf4f25207e9))

## [0.12.2](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.12.1...sui-jsonrpc-v0.12.2)

### ⚙️ Miscellaneous Tasks

- Update itertools to 0.14 - ([bb9404f](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/bb9404fdd4df831bd923ac4b3f977f3c1f2582fd))

## [0.12.1](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.12.0...sui-jsonrpc-v0.12.1)

### ⛰️ Features

- Add `SuiClient::builder` - ([4958dd6](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/4958dd61277c0bc3e422be5a5edc70e0d3cdf2d7))

## [0.12.0](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/sui-jsonrpc-v0.11.0...sui-jsonrpc-v0.12.0)

### ⛰️ Features

- [**breaking**] Update sui-sdk-types to 0.0.2 - ([dead7ff](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/dead7ffe88364166a9de60c48b6da53fe4383e58))

### 🐛 Bug Fixes

- *(sui-jsonrpc)* Set required features for examples - ([c4578b0](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/c4578b08f40533ba3c34169a786287b4d0a3f2d4))

### ⚙️ Miscellaneous Tasks

- *(af-sui-types)* Bump incompat version [propagate] - ([fbf06ff](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/fbf06ff5b383d73297a7595b6a4ca7300bdbfbd2))
- *(af-sui-types)* [**breaking**] Bump to 0.7.0 - ([27e110a](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/27e110a9455d4a1b9c4d9c1a9e4e0c85728a1e96))
- Remove TODOs over which we have no control - ([8629424](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/8629424525f2fdba504740c1cce728a48d8959dc))
- Revert fbf06ff5 - ([8f2567b](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/8f2567b6efd2924092cb5a5a382a5cabeaf7fafd))

## [0.11.0](https://github.com/AftermathFinance/aftermath-sdk-rust/releases/tag/)

### ⛰️ Features

- *(crates)* Add sui-jsonrpc - ([2a5ee5b](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/2a5ee5b0e4440dae59fac6ddbf439a8200c432cb))

### 📚 Documentation

- *(sui-jsonrpc)* Inherit README from crate-level doc - ([400165f](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/400165f5835e909e00a9626883f328ef2ccb1516))

<!-- generated by git-cliff -->
