#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Move types for the core `std` Sui package located at "0x1" onchain.

af_sui_pkg_sdk::sui_pkg_sdk!(std @ "0x1" {
    /// The `ASCII` module defines basic string and char newtypes in Move that verify
    /// that characters are valid ASCII, and that strings consist of only valid ASCII characters.
    module ascii {
        /// The `String` struct holds a vector of bytes that all represent
        /// valid ASCII characters. Note that these ASCII characters may not all
        /// be printable. To determine if a `String` contains only "printable"
        /// characters you should use the `all_characters_printable` predicate
        /// defined in this module.
        #[serde(transparent)]
        struct String has copy, drop, store {
            bytes: vector<u8>,
        }

        /// An ASCII character.
        struct Char has copy, drop, store {
            byte: u8,
        }
    }

    module bit_vector {
        struct BitVector has copy, drop, store {
            length: u64,
            bit_field: vector<bool>,
        }
    }

    /// Defines a fixed-point numeric type with a 32-bit integer part and
    /// a 32-bit fractional part.
    module fixed_point32 {
        /// Define a fixed-point numeric type with 32 fractional bits.
        /// This is just a u64 integer but it is wrapped in a struct to
        /// make a unique type. This is a binary representation, so decimal
        /// values may not be exactly representable, but it provides more
        /// than 9 decimal digits of precision both before and after the
        /// decimal point (18 digits total). For comparison, double precision
        /// floating-point has less than 16 decimal digits of precision, so
        /// be careful about using floating-point to convert these values to
        /// decimal.
        struct FixedPoint32 has copy, drop, store { value: u64 }
    }

    /// This module defines the Option type and its methods to represent and handle an optional value.
    module option {
        /// Abstraction of a value that may or may not be present. Implemented with a vector of size
        /// zero or one because Move bytecode does not have ADTs.
        struct Option<Element> has copy, drop, store {
            vec: vector<Element>
        }
    }

    /// The `string` module defines the `String` type which represents UTF8 encoded strings.
    module string {
        /// Rust: prefer using Rust's native [`String`](std::string::String) which implements
        /// `MoveType`.
        ///
        /// A `String` holds a sequence of bytes which is guaranteed to be in utf8 format.
        struct String has copy, drop, store {
            bytes: vector<u8>,
        }
    }

    /// Functionality for converting Move types into values. Use with care!
    module type_name {
        struct TypeName has copy, drop, store {
            /// String representation of the type. All types are represented
            /// using their source syntax:
            /// "u8", "u64", "bool", "address", "vector", and so on for primitive types.
            /// Struct types are represented as fully qualified type names; e.g.
            /// `00000000000000000000000000000001::string::String` or
            /// `0000000000000000000000000000000a::module_name1::type_name1<0000000000000000000000000000000a::module_name2::type_name2<u64>>`
            /// Addresses are hex-encoded lowercase values of length ADDRESS_LENGTH (16, 20, or 32 depending on the Move platform)
            name: String
        }
    }
});

impl TryFrom<String> for ascii::String {
    type Error = NotAscii;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.is_ascii() {
            return Err(NotAscii);
        }
        Ok(Self {
            bytes: value.bytes().collect::<Vec<_>>().into(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Not an ascii string")]
pub struct NotAscii;

impl TryFrom<ascii::String> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: ascii::String) -> Result<Self, Self::Error> {
        Self::from_utf8(value.bytes.into())
    }
}
