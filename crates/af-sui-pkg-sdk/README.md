<!-- cargo-rdme start -->

Exports the `sui_pkg_sdk!` macro for generating Rust types from Move
source code and implementing relevant `af_move_type` traits.

Automates the conversion of Sui Move types to Rust. The goal is to extract as much information
as possible at compile time about the Move types of a Sui package, generating equivalent Rust
types that:
- are `BCS`-compatible with their on-chain counterparts, so that their contents can be
  deserialized from BCS bytes returned by RPCs
- embed type information based on their location (path) in a Move package + type parameters, so
  that a corresponding type tag can be easily constructed with just the missing information
- use the embedded type information when deserializing a `MoveInstance` to verify the type of
  incoming data, to avoid mistakenly deserializing a different type that has the same BCS bytes

See also:
- `af_move_type`
- `af_move_type_derive` for how the type tag information for a struct is derived from its
  declaration

<div class="warning">
This does not yet support Move 2024 fully.
</div>

# Move types to Rust types

This macro allows callers to *almost* copy and paste Move struct declarations and get equivalent
Rust types. Some additional steps may be necessary however:
- If `phantom` keywords are present, they must be substituted by `!phantom`
- Struct fields should be Rust types. That means they must be in scope. Special Move types like
  `address`, `vector<T>` and `u256` are automatically converted to equivalent Rust types.

The only requirement for a struct field type is that it has the same `BCS` representation as
the Move type for that field. You may use that to your advantage. For instance, if a
`u256` is supposed to be interpreted as a fixed point number, you may define a custom
`FixedP(U256)` type that (de)serializes to/from `u256` bytes but behaves like a fixed point
number.

Additionally, you may add any outter `attributes`, e.g. docs, to structs and their fields.

All `MoveStruct`s created by this macro will have a pretty `Display`
using `tabled` as a backend.

[`af_move_type_derive`]: https://docs.rs/af-move-type-derive/latest/af_move_type_derive/derive.MoveStruct.html
[attributes]: https://doc.rust-lang.org/reference/attributes.html
[BCS]: https://docs.rs/bcs/latest/bcs/

## Examples
```rust
use af_sui_pkg_sdk::sui_pkg_sdk;

sui_pkg_sdk!(package {
    module clearing_house {
        /// Used to dynamically load market objects as needed.
        /// Used to dynamically load traders' position objects as needed.
        struct ClearingHouse<!phantom T> has key {
            id: UID,
            // ...
        }

        /// Stores all deposits from traders for collateral T.
        /// Stores the funds reserved for covering bad debt from untimely
        /// liquidations.
        ///
        /// The Clearing House keeps track of who owns each share of the vault.
        struct Vault<!phantom T> has key, store {
            id: UID,
            collateral_balance: Balance<T>,
            insurance_fund_balance: Balance<T>,
            scaling_factor: u64
        }
    }

    module keys {
        /// Key type for accessing trader position in clearing house.
        struct Position has copy, drop, store {
            account_id: u64,
        }
    }
});
```

Rust types `clearing_house::{ClearingHouse, Vault}` and `keys::Position` will be generated from
the macro call above.

Now suppose we have received a type tag and BCS contents of a Move object from an RPC call. We
can try deserializing it into a `MoveInstance` of one of these generated types
```rust
use af_move_type::{MoveInstance, otw::Otw};
let type_tag: TypeTag;
let base64_bcs: String;

let instance = MoveInstance::<Vault<Otw>>::from_raw_type(
    type_tag,
    &af_sui_types::decode_base64_default(base64_bcs)?
)?;
println!("Coin type {}", instance.type_.t);
```

A few things are happening here:
- `from_raw_type` is checking first that `type_tag` matches the declaration of the struct,
  i.e., it is of the form `_::clearing_house::Vault<_::_::_>`. Anything else will fail
  immediately
- then, it tries to deserialize `clearing_house::Vault` from the BCS bytes

Finally, notice that we're accessing a `type_` field in the Move instance. That's because a
`VaultTypeTag` was automatically generated:
```rust
pub struct VaultTypeTag<T: MoveType> {
    pub address: Address,
    pub t: <T as MoveType>::TypeTag,
}
```
Notice this contains information about the `Vault`'s type tag that couldn't be derived at
compile time, namely, the address of the Move package defining the struct and the concrete type
of the generic type parameter.

One advantage of this type tag over carrying around the generic `StructTag` is that we can
access the type of the OTW directly, while to do so with the latter we'd have to check if
`StructTag::type_params` is not empty every time.

<!-- cargo-rdme end -->

## Future plans & refactors

### Make `tabled` a private dependency

Currently, `tabled` is a [public dependency] because the macro implements the `Tabled` trait for the generated public structs. The only reason that implementation is there, however, is to leverage it in the `Display` implementations for generated structs.

Therefore, it's possible to remove `tabled` from the public API altogether by:
1. making the `pub use tabled;` re-export `#[doc(hidden)]`
1. generating a private `mod $Struct { pub(super) struct DisplayRef<'a> { /* ... */ } }` for each Move struct
1. implementing `tabled::Tabled` for the private struct
1. implementing `Display` for the public struct by constructing `DisplayRef<'_>` internally and using its `Tabled` implementation
1. removing the `move_struct_table_option` function (inline it in the generated code)


[public dependency]: https://rust-lang.github.io/rfcs/3516-public-private-dependencies.html
