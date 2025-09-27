#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Exports the [`sui_pkg_sdk!`](crate::sui_pkg_sdk) macro for generating Rust types from Move
//! source code and implementing relevant [`af_move_type`] traits.
//!
//! Automates the conversion of Sui Move types to Rust. The goal is to extract as much information
//! as possible at compile time about the Move types of a Sui package, generating equivalent Rust
//! types that:
//! - are [BCS]-compatible with their on-chain counterparts, so that their contents can be
//!   deserialized from BCS bytes returned by RPCs
//! - embed type information based on their location (path) in a Move package + type parameters, so
//!   that a corresponding type tag can be easily constructed with just the missing information
//! - use the embedded type information when deserializing a [`MoveInstance`] to verify the type of
//!   incoming data, to avoid mistakenly deserializing a different type that has the same BCS bytes
//!
//! See also:
//! - [`af_move_type`]
//! - [`af_move_type_derive`] for how the type tag information for a struct is derived from its
//!   declaration
//!
//! <div class="warning">
//! This does not yet support Move 2024 fully.
//! </div>
//!
//! # Move types to Rust types
//!
//! This macro allows callers to *almost* copy and paste Move struct declarations and get equivalent
//! Rust types. Some additional steps may be necessary however:
//! - If `phantom` keywords are present, they must be substituted by `!phantom`
//! - Struct fields should be Rust types. That means they must be in scope. Special Move types like
//!   `address`, `vector<T>` and `u256` are automatically converted to equivalent Rust types.
//!
//! The only requirement for a struct field type is that it has the same [BCS] representation as
//! the Move type for that field. You may use that to your advantage. For instance, if a
//! `u256` is supposed to be interpreted as a fixed point number, you may define a custom
//! `FixedP(U256)` type that (de)serializes to/from `u256` bytes but behaves like a fixed point
//! number.
//!
//! Additionally, you may add any outter [attributes], e.g. docs, to structs and their fields.
//!
//! All [`MoveStruct`]s created by this macro will have a pretty [`Display`](::std::fmt::Display)
//! using [`tabled`] as a backend.
//!
//! [`MoveType`]: crate::af_move_type::MoveType
//! [`MoveStruct`]: crate::af_move_type::MoveStruct
//! [`MoveInstance`]: crate::af_move_type::MoveInstance
//! [`af_move_type`]: crate::af_move_type
//! [`af_move_type_derive`]: https://docs.rs/af-move-type-derive/latest/af_move_type_derive/derive.MoveStruct.html
//! [`tabled`]: crate::tabled
//! [attributes]: https://doc.rust-lang.org/reference/attributes.html
//! [BCS]: https://docs.rs/bcs/latest/bcs/
//!
//! ## Examples
//! ```no_run
//! # mod package {
//! # #[derive(
//! #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
//! # )]
//! # pub struct UID {
//! #     id: ID,
//! # }
//! # #[derive(
//! #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
//! # )]
//! # pub struct ID {
//! #     bytes: af_sui_types::ObjectId,
//! # }
//! # #[derive(
//! #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
//! # )]
//! # #[display("Balance")]
//! # pub struct Balance<T> {
//! #     _phantom: std::marker::PhantomData<T>,
//! # }
//! use af_sui_pkg_sdk::sui_pkg_sdk;
//!
//! sui_pkg_sdk!(package {
//!     module clearing_house {
//!         /// Used to dynamically load market objects as needed.
//!         /// Used to dynamically load traders' position objects as needed.
//!         struct ClearingHouse<!phantom T> has key {
//!             id: UID,
//!             // ...
//!         }
//!
//!         /// Stores all deposits from traders for collateral T.
//!         /// Stores the funds reserved for covering bad debt from untimely
//!         /// liquidations.
//!         ///
//!         /// The Clearing House keeps track of who owns each share of the vault.
//!         struct Vault<!phantom T> has key, store {
//!             id: UID,
//!             collateral_balance: Balance<T>,
//!             insurance_fund_balance: Balance<T>,
//!             scaling_factor: u64
//!         }
//!     }
//!
//!     module keys {
//!         /// Key type for accessing trader position in clearing house.
//!         struct Position has copy, drop, store {
//!             account_id: u64,
//!         }
//!     }
//! });
//! # }
//! ```
//!
//! Rust types `clearing_house::{ClearingHouse, Vault}` and `keys::Position` will be generated from
//! the macro call above.
//!
//! Now suppose we have received a type tag and BCS contents of a Move object from an RPC call. We
//! can try deserializing it into a [`MoveInstance`] of one of these generated types
//! ```no_run
//! # mod package {
//! # #[derive(
//! #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
//! # )]
//! # pub struct UID {
//! #     id: ID,
//! # }
//! # #[derive(
//! #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
//! # )]
//! # pub struct ID {
//! #     bytes: af_sui_types::ObjectId,
//! # }
//! # #[derive(
//! #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
//! # )]
//! # #[display("Balance")]
//! # pub struct Balance<T> {
//! #     _phantom: std::marker::PhantomData<T>,
//! # }
//! # use af_sui_pkg_sdk::sui_pkg_sdk;
//! # sui_pkg_sdk!(package {
//! #     module clearing_house {
//! #         struct ClearingHouse<!phantom T> has key {
//! #             id: UID,
//! #         }
//! #         struct Vault<!phantom T> has key, store {
//! #             id: UID,
//! #             collateral_balance: Balance<T>,
//! #             insurance_fund_balance: Balance<T>,
//! #             scaling_factor: u64
//! #         }
//! #     }
//! #     module keys {
//! #         struct Position has copy, drop, store {
//! #             account_id: u64,
//! #         }
//! #     }
//! # });
//! # }
//! # use package::clearing_house::*;
//! # use af_sui_types::{TypeTag};
//! use af_move_type::{MoveInstance, otw::Otw};
//! let type_tag: TypeTag;
//! let base64_bcs: String;
//! # type_tag = TypeTag::Address;
//! # base64_bcs = "".into();
//!
//! let instance = MoveInstance::<Vault<Otw>>::from_raw_type(
//!     type_tag,
//!     &af_sui_types::decode_base64_default(base64_bcs)?
//! )?;
//! println!("Coin type {}", instance.type_.t);
//! # anyhow::Ok(())
//! ```
//!
//! A few things are happening here:
//! - `from_raw_type` is checking first that `type_tag` matches the declaration of the struct,
//!   i.e., it is of the form `_::clearing_house::Vault<_::_::_>`. Anything else will fail
//!   immediately
//! - then, it tries to deserialize `clearing_house::Vault` from the BCS bytes
//!
//! Finally, notice that we're accessing a `type_` field in the Move instance. That's because a
//! `VaultTypeTag` was automatically generated:
//! ```no_run
//! # use af_sui_types::Address;
//! # use af_move_type::MoveType;
//! pub struct VaultTypeTag<T: MoveType> {
//!     pub address: Address,
//!     pub t: <T as MoveType>::TypeTag,
//! }
//! ```
//! Notice this contains information about the `Vault`'s type tag that couldn't be derived at
//! compile time, namely, the address of the Move package defining the struct and the concrete type
//! of the generic type parameter.
//!
//! One advantage of this type tag over carrying around the generic `StructTag` is that we can
//! access the type of the OTW directly, while to do so with the latter we'd have to check if
//! `StructTag::type_params` is not empty every time.

// Re-exports for use in macros
pub use af_move_type::vector::MoveVec;
pub use af_move_type::{HasKey, MoveStruct, MoveType};
pub use af_sui_types::u256::U256;
pub use af_sui_types::{self, Address, ObjectId, TypeTag};
use tabled::grid::config::ColoredConfig;
use tabled::grid::dimension::CompleteDimensionVecRecords;
use tabled::grid::records::vec_records::{Text, VecRecords};
use tabled::settings::panel::Header;
use tabled::settings::style::Style;
use tabled::settings::{Rotate, Settings, TableOption};
pub use tabled::{self, Table, Tabled};
pub use {af_move_type, derive_new, serde};

/// Generate [`MoveStruct`] implementations from Move source code.
///
/// See the top-level [crate] documentation for usage and examples.
#[macro_export]
macro_rules! sui_pkg_sdk {
    // =========================================================================
    //  Move package
    // =========================================================================
    ($package:ident$(@$address:literal)? {
        $($tt:tt)*
    }) => {
        $crate::sui_pkg_sdk!(@Modules $(@$address)? $($tt)*);
    };

    // =========================================================================
    //  Move modules
    // =========================================================================
    (@Modules @$address:literal
        $(
            $(#[$meta:meta])*
            module $module:ident {
                $($tt:tt)*
            }
        )*
    ) => {
        $(
            $crate::sui_pkg_sdk!(@Module $(#[$meta])* $address::$module
                $($tt)*
            );
        )*
    };

    (@Modules
        $(
            $(#[$meta:meta])*
            module $module:ident {
                $($tt:tt)*
            }
        )*
    ) => {
        $(
            $crate::sui_pkg_sdk!(@Module $(#[$meta])* $module
                $($tt)*
            );
        )*
    };

    (@Module $(#[$meta:meta])* $($address:literal::)?$module:ident
        $($tt:tt)*
    ) => {
        #[allow(clippy::too_many_arguments)]
        $(#[$meta])*
        pub mod $module {
            #[allow(unused_imports)]
            use super::*;

            #[allow(non_camel_case_types, unused)]
            type address = $crate::Address;
            #[allow(non_camel_case_types, unused)]
            type u256 = $crate::U256;
            #[allow(non_camel_case_types, unused)]
            type vector<T> = $crate::MoveVec<T>;

            $crate::sui_pkg_sdk!(@ModuleMembers $($address::)?$module {
                $($tt)*
            });
        }
    };

    // =========================================================================
    //  No more module content to parse
    // =========================================================================
    (@ModuleMembers $($address:literal::)?$module:ident { }) => { };

    // =========================================================================
    //  Braced structs (i.e., with named fields):
    //  - derive new
    //  - impl HasKey if 'key' ability present
    // =========================================================================
    (@ModuleMembers $($address:literal::)?$module:ident {
        $(#[$meta:meta])* // attributes (forwarded)
        $(public $( ($_scope:ident) )? )? // visibility (ignored)
        struct $Struct:ident
        $(<$($(!$phantom:ident)? $T:ident$(: $_:ident $(+ $__:ident)*)?),*>)? // type params
        $(has $($ability:ident),+)? // abilities (transformed into traits)
        {
            $($struct_content:tt)+
        }

        $($rest:tt)*
    }) => {
        $crate::sui_pkg_sdk!(@Struct
            // added attributes
            #[derive($crate::derive_new::new)]
            #[move_(module=$module)]
            $(#[move_(address=$address)])?

            $(#[$meta])* // forwarded attributes

            $Struct$(<$($T),*>)? // ident and type params for struct declaration
            [$($($($phantom)? $T,)*)?] // type params for potentially PhantomData
            ($($struct_content)+) // contents
            -> {}
        );

        $crate::sui_pkg_sdk!(@abilities
            $Struct$(<$($T),*>)? [$($($ability,)+)?]
        );

        $crate::sui_pkg_sdk!(@ModuleMembers $($address::)?$module {
            $($rest)*
        });
    };

    // =========================================================================
    //  Tuple structs:
    //  - derive new
    //  - impl HasKey if 'key' ability present
    // =========================================================================
    (@ModuleMembers $($address:literal::)?$module:ident {
        $(#[$meta:meta])* // attributes (forwarded)
        $(public $( ($_scope:ident) )? )? // visibility (ignored)
        struct $Struct:ident
        $(<$($(!$phantom:ident)? $T:ident$(: $_:ident $(+ $__:ident)*)?),*>)? // type params
        ($( $struct_content:tt )+) // contents
        $(has $($ability:ident),+)?; // abilities

        $($rest:tt)*
    }) => {
        $crate::sui_pkg_sdk!(@Struct
            #[derive($crate::derive_new::new)]
            #[move_(module=$module)]
            $(#[move_(address=$address)])?
            $(#[$meta])*
            $Struct$(<$($T),*>)?
            [$($($($phantom)? $T,)*)?]
            ($($struct_content)+)
            -> ()
        );

        $crate::sui_pkg_sdk!(@abilities
            $Struct$(<$($T),*>)? [$($($ability,)+)?]
        );

        $crate::sui_pkg_sdk!(@ModuleMembers $($address::)?$module {
            $($rest)*
        });
    };

    // =========================================================================
    //  Empty braced struct:
    //  - add dummy field
    //  - custom new
    //  - impl Default
    //  - skip ability parsing since empty structs can't have the 'key' ability
    // =========================================================================
    (@ModuleMembers $($address:literal::)?$module:ident {
        $(#[$meta:meta])* // attributes (forwarded)
        $(public $( ($_scope:ident) )? )? // visibility (ignored)
        struct $Struct:ident$(<$(!phantom $T:ident),*>)?
        $(has $($ability:ident),+)? // abilities, ignored for now
        {}

        $($rest:tt)*
    }) => {
        $crate::sui_pkg_sdk!(@Struct
            #[move_(module=$module)]
            $(#[move_(address=$address)])?
            $(#[$meta])*
            $Struct$(<$($T),*>)?
            [$($(phantom $T,)*)?]
            ()
            -> { dummy_field: bool, }
        );

        impl$(<$($T: $crate::MoveType),*>)? $Struct$(<$($T),*>)? {
            pub fn new() -> Self {
                Self {
                    dummy_field: false,
                    $($( // Any type parameter in an empty struct must be a phantom one
                        $T: ::std::marker::PhantomData::<$T>,
                    )*)?
                }
            }
        }

        // To address `clippy::new_without_default`
        impl$(<$($T: $crate::MoveType),*>)? ::std::default::Default for $Struct$(<$($T),*>)? {
            fn default() -> Self {
                Self::new()
            }
        }

        $crate::sui_pkg_sdk!(@ModuleMembers $($address::)?$module {
            $($rest)*
        });
    };

    // =========================================================================
    //  Empty tuple struct:
    //  - add dummy field
    //  - custom new
    //  - impl Default
    //  - skip ability parsing since empty structs can't have the 'key' ability
    // =========================================================================
    (@ModuleMembers $($address:literal::)?$module:ident {
        $(#[$meta:meta])* // attributes (forwarded)
        $(public $( ($_scope:ident) )? )? // visibility (ignored)
        struct $Struct:ident$(<$(!phantom $T:ident),*>)?
        ()
        $(has $($ability:ident),+)? // abilities, ignored for now
        ;

        $($rest:tt)*
    }) => {
        $crate::sui_pkg_sdk!(@Struct
            #[move_(module=$module)]
            $(#[move_(address=$address)])?
            $(#[$meta])*
            $Struct$(<$($T),*>)?
            [$($(phantom $T,)*)?]
            ()
            -> ( bool, )
        );

        impl$(<$($T: $crate::MoveType),*>)? $Struct$(<$($T),*>)? {
            pub fn new() -> Self {
                Self(
                    false,
                    $( $( // Any type parameter in an empty struct must be a phantom one
                        ::std::marker::PhantomData::<$T>,
                    )* )?
                )
            }
        }

        // To address `clippy::new_without_default`
        impl$(<$($T: $crate::MoveType),*>)? ::std::default::Default for $Struct$(<$($T),*>)? {
            fn default() -> Self {
                Self::new()
            }
        }

        $crate::sui_pkg_sdk!(@ModuleMembers $($address::)?$module {
            $($rest)*
        });
    };

    // =========================================================================
    //  Braced struct builder
    //
    //  God bless this StackOverflow response:
    //  https://stackoverflow.com/a/53582890
    // =========================================================================
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        []
        ()
        -> { $($result:tt)* }
    ) => {
        #[derive(
            $crate::MoveStruct,
            $crate::serde::Deserialize,
            $crate::serde::Serialize,
            $crate::Tabled,
            Clone,
            Debug,
            PartialEq,
            Eq,
            Hash,
        )]
        $(#[$meta])*
        #[move_(crate = ::af_sui_pkg_sdk::af_move_type)]
        #[serde(crate = "::af_sui_pkg_sdk::serde")]
        #[serde(bound(deserialize = ""))]
        #[tabled(crate = "::af_sui_pkg_sdk::tabled")]
        #[allow(non_snake_case)]
        pub struct $Struct$(<$($G: $crate::MoveType),*>)? {
            $($result)*
        }

        impl$(<$($G: $crate::MoveType),*>)? ::std::fmt::Display for $Struct$(<$($G),*>)? {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let settings = $crate::move_struct_table_option(stringify!($Struct));
                let mut table = $crate::Table::new([self]);
                table.with(settings);
                write!(f, "{}", table)
            }
        }
    };

    // -------------------------------------------------------------------------
    // phantom type parameter: a `PhantomData` field is added to the struct
    // -------------------------------------------------------------------------
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        [phantom $T:ident, $($rest:tt)*]
        ($($fields:tt)*)
        -> { $($result:tt)* }
    ) => {
        $crate::sui_pkg_sdk!(@Struct
            $(#[$meta])*
            $Struct$(<$($G),*>)?
            [$($rest)*]
            ($($fields)*)
            -> {
                $($result)*

                #[tabled(skip)]
                #[serde(skip_deserializing, skip_serializing, default)]
                $T: ::std::marker::PhantomData<$T>,
            }
        );
    };

    // -------------------------------------------------------------------------
    // generic type parameter: no influence in struct contents
    // -------------------------------------------------------------------------
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        [$_T:ident, $($rest:tt)*]
        ($($fields:tt)*)
        -> { $($result:tt)* }
    ) => {
        $crate::sui_pkg_sdk!(@Struct
            $(#[$meta])*
            $Struct$(<$($G),*>)?
            [$($rest)*]
            ($($fields)*)
            -> { $($result)* }
        );
    };

    // #[tabled(format("{}.{}", self.id, self.name))]
    // -------------------------------------------------------------------------
    // option field type
    // -------------------------------------------------------------------------
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        []
        ($(#[$fmeta:meta])* $field:ident: Option<$type:ty> $(, $($rest:tt)*)?)
        -> { $($result:tt)* }
    ) => {
        $crate::sui_pkg_sdk!(@Struct
            $(#[$meta])*
            $Struct$(<$($G),*>)?
            []
            ($($($rest)*)?)
            -> {
                $($result)*
                $(#[$fmeta])*
                #[tabled(format(
                    "{}",
                    self.$field
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "None".into())
                ))]
                pub $field: Option<$type>,
            }
        );
    };


    // -------------------------------------------------------------------------
    // generic field type
    // -------------------------------------------------------------------------
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        []
        ($(#[$fmeta:meta])* $field:ident: $type:ty $(, $($rest:tt)*)?)
        -> { $($result:tt)* }
    ) => {
        $crate::sui_pkg_sdk!(@Struct
            $(#[$meta])*
            $Struct$(<$($G),*>)?
            []
            ($($($rest)*)?)
            -> {
                $($result)*
                $(#[$fmeta])* pub $field: $type,
            }
        );
    };

    // =========================================================================
    //  Tuple struct builder
    // =========================================================================
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        []
        ()
        -> ( $($result:tt)* )
    ) => {
        #[derive(
            $crate::MoveStruct,
            $crate::serde::Deserialize,
            $crate::serde::Serialize,
            $crate::Tabled,
            Clone,
            Debug,
            PartialEq,
            Eq,
            Hash,
        )]
        $(#[$meta])*
        #[move_(crate = ::af_sui_pkg_sdk::af_move_type)]
        #[serde(crate = "::af_sui_pkg_sdk::serde")]
        #[serde(bound(deserialize = ""))]
        #[tabled(crate = "::af_sui_pkg_sdk::tabled")]
        #[allow(non_snake_case)]
        pub struct $Struct$(<$($G: $crate::MoveType),*>)? (
            $($result)*
        );

        impl$(<$($G: $crate::MoveType),*>)? ::std::fmt::Display for $Struct$(<$($G),*>)? {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let settings = $crate::move_struct_table_option(stringify!($Struct));
                let mut table = $crate::Table::new([self]);
                table.with(settings);
                write!(f, "{}", table)
            }
        }
    };

    // -------------------------------------------------------------------------
    // phantom type parameter: a `PhantomData` field is added to the struct
    // -------------------------------------------------------------------------
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        [phantom $T:ident, $($rest:tt)*]
        ($($fields:tt)*)
        -> ( $($result:tt)* )
    ) => {
        $crate::sui_pkg_sdk!(@Struct
            $(#[$meta])*
            $Struct$(<$($G),*>)?
            [$($rest)*]
            ($($fields)*)
            -> (
                $($result)*

                #[tabled(skip)]
                #[serde(skip_deserializing, skip_serializing, default)]
                ::std::marker::PhantomData<$T>,
            )
        );
    };

    // -------------------------------------------------------------------------
    // generic type parameter: no influence in struct contents
    // -------------------------------------------------------------------------
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        [$_T:ident, $($rest:tt)*]
        ($($fields:tt)*)
        -> ( $($result:tt)* )
    ) => {
        $crate::sui_pkg_sdk!(@Struct
            $(#[$meta])*
            $Struct$(<$($G),*>)?
            [$($rest)*]
            ($($fields)*)
            -> ( $($result)* )
        );
    };

    // -------------------------------------------------------------------------
    // generic field type
    // -------------------------------------------------------------------------
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        []
        ($(#[$fmeta:meta])* $type:ty $(, $($rest:tt)*)?)
        -> ( $($result:tt)* )
    ) => {
        $crate::sui_pkg_sdk!(@Struct
            $(#[$meta])*
            $Struct$(<$($G),*>)?
            []
            ($($($rest)*)?)
            -> (
                $($result)*
                $(#[$fmeta])* pub $type,
            )
        );
    };

    // =========================================================================
    //  Abilities parser
    // =========================================================================
    (@abilities
        $Struct:ident$(<$($G:ident),*>)?
        []
    ) => {};

    (@abilities
        $Struct:ident$(<$($G:ident),*>)?
        [key, $($rest:tt)*]
    ) => {
        impl$(<$($G: $crate::MoveType),*>)? $crate::HasKey for $Struct$(<$($G),*>)? {
            fn object_id(&self) -> $crate::ObjectId {
                self.id.id.bytes
            }
        }

        $crate::sui_pkg_sdk!(@abilities
            $Struct$(<$($G),*>)? [$($rest)*]
        );
    };

    (@abilities
        $Struct:ident$(<$($G:ident),*>)?
        [$other:ident, $($rest:tt)*]
    ) => {
        $crate::sui_pkg_sdk!(@abilities
            $Struct$(<$($G),*>)? [$($rest)*]
        );
    };
}

// =========================================================================
//  Tabled display helpers
// =========================================================================

pub fn move_struct_table_option<S: AsRef<str>>(
    name: S,
) -> impl for<'a> TableOption<VecRecords<Text<String>>, ColoredConfig, CompleteDimensionVecRecords<'a>>
{
    Settings::default()
        .with(Rotate::Left)
        .with(Rotate::Top)
        .with(Style::rounded())
        .with(Header::new(name))
}
