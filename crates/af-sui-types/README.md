# af-sui-types

## About

<!-- cargo-rdme start -->

Aftermath's extensions to `sui_sdk_types`.

Includes some types and constants from the original `sui_types` and `move_core_types` that
are not present in `sui_sdk_types`. This crate also re-exports a lot of the types in
`sui_sdk_types`.

This crate was originally conceived with the following objectives:
- `serde` compatibility with the full Sui checkpoint data
- avoiding dynamic error types so that callers could match against errors and react accordingly
- using a minimal set of dependencies
- [SemVer](https://doc.rust-lang.org/cargo/reference/semver.html) compatibility

<div class="warning">

The long-term plan is to deprecate most of this in favor of `sui_sdk_types`. However, there
are some types in that crate that don't expose all of the attributes/methods we need yet.

</div>

[`serde`]: https://docs.rs/serde/latest/serde/index.html
[`sui_types`]: https://mystenlabs.github.io/sui/sui_types/index.html
[`move_core_types`]: https://github.com/MystenLabs/sui/tree/main/external-crates/move/crates/move-core-types
[`sui_sdk_types`]: https://docs.rs/sui-sdk-types/latest/sui_sdk_types/

<!-- cargo-rdme end -->
