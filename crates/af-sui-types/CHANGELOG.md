# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

## [0.7.5](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/af-sui-types-v0.7.4...af-sui-types-v0.7.5)

### ⛰️ Features

- ObjectArg::SYSTEM_STATE_IMM - ([9552e36](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/9552e368e8a63653eddeb6531ab29856381a51db))


## [0.7.3](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/af-sui-types-v0.7.2...af-sui-types-v0.7.3)

### 📚 Documentation

- Standardize changelogs - ([383b40d](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/383b40d75c38f637aafe06438673f71e1c57d432))

### ⚙️ Miscellaneous Tasks

- Re-export TypeParseError - ([6e930d6](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/6e930d6407b90dbfdd8667a68c3e94a73433daca))


## [0.7.2](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/af-sui-types-v0.7.1...af-sui-types-v0.7.2)

### 📚 Documentation

- Regenerate changelogs from scratch - ([288008f](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/288008f5b60193ea34b765d8ad605cf4f25207e9))

## [0.7.1](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/af-sui-types-v0.7.0...af-sui-types-v0.7.1)

### ⛰️ Features

- *(af-sui-types)* Export `TransactionFromBase64Error` - ([3c1640b](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/3c1640bd781d158d518572375d1855cc43b58a94))
- *(af-sui-types)* Use string for `TransactionFromBase64Error::Base64` - ([3043e34](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/3043e3402faa2c4503235dc3505667b6e20a8858))
- Deprecate `TransactionData::decode_base64` - ([f74b7d1](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/f74b7d150e6e0d94f10abcfd225108948235ccf2))

## [0.7.0](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/af-sui-types-v0.6.2...af-sui-types-v0.7.0)

### ⛰️ Features

- *(af-sui-types)* [**breaking**] Replace TransactionEvents - ([45cdf26](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/45cdf263b5b772cffd6f040b877207c5bc21ed92))
- *(af-sui-types)* [**breaking**] Remove deprecated aliases - ([af7d9ea](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/af7d9eaad7948ef7724edce36b1ee51b83005701))
- *(af-sui-types)* [**breaking**] Remove deprecated methods - ([eafc15b](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/eafc15bd3b28f03333f9bb6a5ebcb6972c86790e))
- *(af-sui-types)* Add const-ructors for Address and ObjectId - ([84e26c7](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/84e26c7dc9050c81223978edb22a2a95f6d60192))
- *(af-sui-types)* Add `From<sui_sdk_types::Owner> for Owner` - ([4bab2f2](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/4bab2f2a8e794b8945f321816018c72e04b5ed36))
- *(af-sui-types)* [**breaking**] Replace TransactionEffects - ([a22c555](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/a22c5558f9062c4a5111dfb1ff65ce98b9c169e1))
- *(af-sui-types)* [**breaking**] Remove deps on ethnum,num - ([148a423](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/148a423aa49532956fd5ce2b2642b009bcf029bd))
- *(af-sui-types)* [**breaking**] Remove dependency on arbitrary - ([fdcf57f](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/fdcf57fc14aac72e234c5790f5a98743ecb5ca62))
- *(af-sui-types)* [**breaking**] Replace `MovePackage,TypeOrigin,UpgradeInfo` - ([d8a6da4](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/d8a6da4d92c6dbbb4352d10f2aba2bad155fa0cf))
- *(af-sui-types)* [**breaking**] Return sui sdk types in TransactionEffectsAPI - ([72fcb66](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/72fcb66746bc9c10a73597fd93c5ccf971dc62d7))
- [**breaking**] Update sui-sdk-types to 0.0.2 - ([dead7ff](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/dead7ffe88364166a9de60c48b6da53fe4383e58))

### 🚜 Refactor

- *(af-sui-types)* Simplify deps, requiring fewer features - ([14f5dbd](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/14f5dbdff78e02ed047f7fcf3e8694110441f709))
- *(af-sui-types)* [**breaking**] Have `Owner::ObjectOwner` contain an `ObjectId` - ([3e7cd55](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/3e7cd5537fb23156323c9755c818c1d21e2a4ecc))
- *(af-sui-types)* [**breaking**] `Option`-returning `Owner` methods - ([e028ee6](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/e028ee63de43f9a1e3fcb551eece699fb9335aea))

### 📚 Documentation

- *(af-sui-types)* Comment on TransactionEffectsAPI - ([ee2e316](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/ee2e316e95a13c11b3b739c45645952d8907eb00))

### ⚙️ Miscellaneous Tasks

- *(af-sui-types)* Remove `Owner` TODOs - ([bd8c6a8](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/bd8c6a8caf2f847d7cfb3ee620c3c954553ae4ba))
- *(af-sui-types)* Bump incompat version [propagate] - ([fbf06ff](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/fbf06ff5b383d73297a7595b6a4ca7300bdbfbd2))
- *(af-sui-types)* [**breaking**] Bump to 0.7.0 - ([27e110a](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/27e110a9455d4a1b9c4d9c1a9e4e0c85728a1e96))
- Revert fbf06ff5 - ([8f2567b](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/8f2567b6efd2924092cb5a5a382a5cabeaf7fafd))

## [0.6.2](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/af-sui-types-v0.6.1...af-sui-types-v0.6.2)

### ⛰️ Features

- *(crates)* Add remaining crates (#2) - ([5d2dae1](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/5d2dae1392de8ed6a5af63a0e559bd3416112b35))

### ⚙️ Miscellaneous Tasks

- *(main)* Release af-sui-types 0.6.2 (#33) - ([5f069fa](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/5f069fadad974ecdca26bde7fa8d754e68037f92))
- Remove cyclical dev dependencies - ([08d9a17](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/08d9a1710fb56c3a58663051eecf29a18e91594b))

## [0.6.1](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/af-sui-types-v0.6.0...af-sui-types-v0.6.1)

### 📚 Documentation

- *(af-sui-types)* Extract development out of the README - ([c296377](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/c29637727bb3309ba45a4674e55ec7d2d3f97074))

### ⚙️ Miscellaneous Tasks

- Release - ([4fa6779](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/4fa67794d13f77da7b4d516fe22f83afa025f541))

## [0.6.0](https://github.com/AftermathFinance/aftermath-sdk-rust/releases/tag/)

### ⛰️ Features

- *(crates)* Add af-sui-types - ([189ec3a](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/189ec3a493e581a1bdef369c661a12816a742a56))

<!-- generated by git-cliff -->
