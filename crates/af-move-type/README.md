<!-- cargo-rdme start -->

Defines the core standard for representing Move types off-chain and their type tags.

The core items are `MoveType` and `MoveTypeTag`. These
are useful trait bounds to use when dealing with generic off-chain Move type representations.
They are implemented for the primitive types that correspond to Move's primitives
(integers/bool). Also included is `MoveVec`, corresponding to `vector`
and defining a pretty `Display`.

For Move structs (objects), `MoveStruct` should be used as it has an
associated `MoveStructTag`. The
[`MoveStruct`](af_move_type_derive::MoveStruct) derive macro is exported for automatically
creating a `MoveStructTag` implementation from normal Rust struct declarations.

A specific instance of a Move type is represented by `MoveInstance`.

<!-- cargo-rdme end -->
