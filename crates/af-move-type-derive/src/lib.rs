//! Derive macros for traits defined in `af-move-type`.

mod move_struct;

use move_struct::impl_move_struct;
use proc_macro::TokenStream;

/// Derives `af_move_type` trait implementations for a type representing a Move struct.
///
/// Creates the `_TypeTag` struct related to the struct being annotated, with conversion traits
/// between the former and the dynamic `StructTag` type, with errors like 'expected module to be x'
/// or 'expected struct name to be y', if we know those things at compile time (see the
/// [Attributes](#attributes) section for configurations around those checks).
///
/// # Attributes
///
/// - `#[move_(crate = ...)]`: sets the path of the `af_move_type` crate, which can be useful if
///   using this inside other macros.
/// - `#[move_(address = "...")]`: sets a static package address for the derived `MoveTypeTag`.
///   Deserialization of the latter will fail if the package addresses do not match.
/// - `#[move_(module = "...")]`: sets a static module name for the derived `MoveTypeTag`.
///   Deserialization of the latter will fail if the module names do not match.
/// - `#[move_(nameless)]`: make the struct name dynamic for the derived `MoveTypeTag`. Upon the
///   deserializing the latter, any Move struct name will be accepted. Otherwise, deserialization
///   will fail if the incoming struct name is not equal to the Rust struct's name.
///
/// # `MoveTypeTag` derivation
///
/// For a struct `Name<T: MoveType>`, the macro will create a `NameTypeTag` struct with fields:
/// - `address: Address`, unless the `#[move_(address = "...")]` attribute is present
/// - `module: Identifier`, unless the `#[move_(module = "...")]` attribute is present
/// - `name: Identifier` only if the `#[move_(nameless)]` attribute is present
/// - `t: <T as MoveType>::TypeTag`
///
/// The macro will also create custom `Into<StructTag>`, `Into<TypeTag>`, `TryFrom<StructTag>`,
/// `TryFrom<TypeTag>`, `Display` and `FromStr` impls for `NameTypeTag`.
///
/// # Derived traits
///
/// - `af_move_type::MoveStruct`
/// - `af_move_type::StaticAddress` if `#[move_(address = "...")]` is specified
/// - `af_move_type::StaticModule` if `#[move_(module = "...")]` is specified
/// - `af_move_type::StaticName` if `#[move_(nameless)]` was **not** specified
/// - `af_move_type::StaticTypeParams`
/// - `af_move_type::StaticStructTag` if applicable
/// - `af_move_type::StaticTypeTag` if applicable
///
#[proc_macro_derive(MoveStruct, attributes(move_))]
pub fn move_struct_derive_macro(item: TokenStream) -> TokenStream {
    impl_move_struct(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
