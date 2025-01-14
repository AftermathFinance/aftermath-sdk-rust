# Changelog

## [0.7.0](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/af-sui-types-v0.6.2...af-sui-types-v0.7.0) (2025-01-14)


### ⚠ BREAKING CHANGES

* **af-sui-types:** bump to 0.7.0
* **af-sui-types:** return sui sdk types in TransactionEffectsAPI
* **af-sui-types:** `Option`-returning `Owner` methods
* **af-sui-types:** have `Owner::ObjectOwner` contain an `ObjectId`
* **af-sui-types:** replace `MovePackage,TypeOrigin,UpgradeInfo`
* **af-sui-types:** remove dependency on arbitrary
* **af-sui-types:** remove deps on ethnum,num
* **af-sui-types:** replace TransactionEffects
* **af-sui-types:** remove deprecated methods
* **af-sui-types:** remove deprecated aliases
* **af-sui-types:** replace TransactionEvents
* update sui-sdk-types to 0.0.2

### Features

* **af-sui-types:** add `From&lt;sui_sdk_types::Owner&gt; for Owner` ([4bab2f2](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/4bab2f2a8e794b8945f321816018c72e04b5ed36))
* **af-sui-types:** add const-ructors for Address and ObjectId ([84e26c7](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/84e26c7dc9050c81223978edb22a2a95f6d60192))
* **af-sui-types:** remove dependency on arbitrary ([fdcf57f](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/fdcf57fc14aac72e234c5790f5a98743ecb5ca62))
* **af-sui-types:** remove deprecated aliases ([af7d9ea](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/af7d9eaad7948ef7724edce36b1ee51b83005701))
* **af-sui-types:** remove deprecated methods ([eafc15b](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/eafc15bd3b28f03333f9bb6a5ebcb6972c86790e))
* **af-sui-types:** remove deps on ethnum,num ([148a423](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/148a423aa49532956fd5ce2b2642b009bcf029bd))
* **af-sui-types:** replace `MovePackage,TypeOrigin,UpgradeInfo` ([d8a6da4](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/d8a6da4d92c6dbbb4352d10f2aba2bad155fa0cf))
* **af-sui-types:** replace TransactionEffects ([a22c555](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/a22c5558f9062c4a5111dfb1ff65ce98b9c169e1))
* **af-sui-types:** replace TransactionEvents ([45cdf26](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/45cdf263b5b772cffd6f040b877207c5bc21ed92))
* **af-sui-types:** return sui sdk types in TransactionEffectsAPI ([72fcb66](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/72fcb66746bc9c10a73597fd93c5ccf971dc62d7))
* update sui-sdk-types to 0.0.2 ([dead7ff](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/dead7ffe88364166a9de60c48b6da53fe4383e58))


### Miscellaneous Chores

* **af-sui-types:** bump to 0.7.0 ([27e110a](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/27e110a9455d4a1b9c4d9c1a9e4e0c85728a1e96))


### Code Refactoring

* **af-sui-types:** `Option`-returning `Owner` methods ([e028ee6](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/e028ee63de43f9a1e3fcb551eece699fb9335aea))
* **af-sui-types:** have `Owner::ObjectOwner` contain an `ObjectId` ([3e7cd55](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/3e7cd5537fb23156323c9755c818c1d21e2a4ecc))

## [0.6.2](https://github.com/AftermathFinance/aftermath-sdk-rust/compare/af-sui-types-v0.6.1...af-sui-types-v0.6.2) (2025-01-14)


### Features

* **crates:** add remaining crates ([#2](https://github.com/AftermathFinance/aftermath-sdk-rust/issues/2)) ([5d2dae1](https://github.com/AftermathFinance/aftermath-sdk-rust/commit/5d2dae1392de8ed6a5af63a0e559bd3416112b35))
