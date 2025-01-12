#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Macro for generating [`MoveStruct`] implementations from Move source code.
//!
//! See [`sui_pkg_sdk!`](crate::sui_pkg_sdk).

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

/// Automates the conversion of Sui Move types to Rust.
///
/// **NOTE**: this does not yet support Move 2024 fully.
///
/// # Move types to Rust types
///
/// This macro allows callers to *almost* copy and paste Move struct declarations and get equivalent
/// Rust types. Some additional steps may be necessary however:
/// - If `phantom` keywords are present, they must be substituted by `!phantom`
/// - Struct fields should be Rust types. That means they must be in scope. Common cases like
///   `address`, `vector<T>` and `u256` are automatically handled.
///
/// You may use the second point above to leverage custom deserializations. For instance, if a
/// `u256` is supposed to be interpreted as a fixed point number, you may define a custom
/// [`MoveType`] that (de)serializes to/from `u256` but behaves like a fixed point number.
///
/// Additionally, you may add any outter attributes, e.g. docs, to structs and their fields.
///
/// All [`MoveStruct`]s created by this macro will have a pretty [`Display`](std::fmt::Display)
/// using [`tabled`] as a backend.
///
/// [`MoveType`]: crate::af_move_type::MoveType
///
/// ## Examples
/// ```no_run
/// # mod package {
/// # #[derive(
/// #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
/// # )]
/// # pub struct UID {
/// #     id: ID,
/// # }
/// # #[derive(
/// #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
/// # )]
/// # pub struct ID {
/// #     bytes: af_sui_types::ObjectId,
/// # }
/// # #[derive(
/// #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
/// # )]
/// # #[display("Balance")]
/// # pub struct Balance<T> {
/// #     _phantom: std::marker::PhantomData<T>,
/// # }
/// // use sui_framework_sdk::{object::UID, balance::Balance};
/// use af_sui_pkg_sdk::sui_pkg_sdk;
///
/// sui_pkg_sdk!(package {
///     module clearing_house {
///         /// Used to dynamically load market objects as needed.
///         /// Used to dynamically load traders' position objects as needed.
///         struct ClearingHouse<!phantom T> has key {
///             id: UID,
///             // ...
///         }
///
///         /// Stores all deposits from traders for collateral T.
///         /// Stores the funds reserved for covering bad debt from untimely
///         /// liquidations.
///         ///
///         /// The Clearing House keeps track of who owns each share of the vault.
///         struct Vault<!phantom T> has key, store {
///             id: UID,
///             collateral_balance: Balance<T>,
///             insurance_fund_balance: Balance<T>,
///             scaling_factor: u64
///         }
///     }
///
///     module keys {
///         /// Key type for accessing trader position in clearing house.
///         struct Position has copy, drop, store {
///             account_id: u64,
///         }
///     }
/// });
/// # }
/// ```
///
/// Should generate:
/// ```no_run
/// # mod package {
/// # use af_sui_types::ObjectId;
/// # use af_move_type::{MoveStruct, MoveType, HasKey};
/// # #[derive(
/// #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
/// # )]
/// # pub struct UID {
/// #     id: ID,
/// # }
/// # #[derive(
/// #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
/// # )]
/// # pub struct ID {
/// #     bytes: af_sui_types::ObjectId,
/// # }
/// # #[derive(
/// #     serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
/// # )]
/// # #[display("Balance")]
/// # pub struct Balance<T> {
/// #     _phantom: std::marker::PhantomData<T>,
/// # }
/// pub mod clearing_house {
///     use super::*;
///
///     /// Used to dynamically load market objects as needed.
///     /// Used to dynamically load traders' position objects as needed.
///     #[derive(
///         MoveStruct,
///         serde::Deserialize,
///         serde::Serialize,
///         Clone,
///         Debug,
///         PartialEq,
///         Eq,
///         Hash,
///         tabled::Tabled,
///     )]
///     #[derive(derive_new::new)]
///     #[move_(module=clearing_house)]
///     #[serde(bound(deserialize = ""))]
///     #[allow(non_snake_case)]
///     pub struct ClearingHouse<T: MoveType> {
///         pub id: UID,
///         #[serde(skip_deserializing, skip_serializing, default)]
///         #[tabled(skip)]
///         T: ::std::marker::PhantomData<T>,
///     }
///
///     impl<T: MoveType> std::fmt::Display for ClearingHouse<T> {
///         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///             let settings = af_sui_pkg_sdk::move_struct_table_option("ClearingHouse");
///             let mut table = tabled::Table::new([self]);
///             table.with(settings);
///             f.write_fmt(format_args!("{0}", table))
///         }
///     }
///
///     impl<T: MoveType> HasKey for ClearingHouse<T> {
///         fn object_id(&self) -> ObjectId {
///             self.id.id.bytes
///         }
///     }
///
///     /// Stores all deposits from traders for collateral T.
///     /// Stores the funds reserved for covering bad debt from untimely
///     /// liquidations.
///     ///
///     /// The Clearing House keeps track of who owns each share of the vault.
///     #[derive(
///         MoveStruct,
///         serde::Deserialize,
///         serde::Serialize,
///         Clone,
///         Debug,
///         PartialEq,
///         Eq,
///         Hash,
///         tabled::Tabled,
///     )]
///     #[derive(derive_new::new)]
///     #[move_(module=clearing_house)]
///     #[serde(bound(deserialize = ""))]
///     #[allow(non_snake_case)]
///     pub struct Vault<T: MoveType> {
///         pub id: UID,
///         pub collateral_balance: Balance<T>,
///         pub insurance_fund_balance: Balance<T>,
///         pub scaling_factor: u64,
///         #[serde(skip_deserializing, skip_serializing, default)]
///         #[tabled(skip)]
///         T: ::std::marker::PhantomData<T>,
///     }
///
///     impl<T: MoveType> std::fmt::Display for Vault<T> {
///         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///             let settings = af_sui_pkg_sdk::move_struct_table_option("Vault");
///             let mut table = tabled::Table::new([self]);
///             table.with(settings);
///             f.write_fmt(format_args!("{0}", table))
///         }
///     }
///
///     impl<T: MoveType> HasKey for Vault<T> {
///         fn object_id(&self) -> ObjectId {
///             self.id.id.bytes
///         }
///     }
/// }
///
/// pub mod keys {
///     use super::*;
///
///     /// Key type for accessing trader position in clearing house.
///     #[derive(
///         MoveStruct,
///         serde::Deserialize,
///         serde::Serialize,
///         Clone,
///         Debug,
///         PartialEq,
///         Eq,
///         Hash,
///         tabled::Tabled,
///     )]
///     #[derive(derive_new::new)]
///     #[move_(module=keys)]
///     #[serde(bound(deserialize = ""))]
///     #[allow(non_snake_case)]
///     pub struct Position {
///         pub account_id: u64,
///     }
///
///     impl std::fmt::Display for Position {
///         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///             let settings = af_sui_pkg_sdk::move_struct_table_option("Position");
///             let mut table = tabled::Table::new([self]);
///             table.with(settings);
///             f.write_fmt(format_args!("{0}", table))
///         }
///     }
/// }
/// # }
/// ```
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
    //  General structs:
    //  - derive new
    //  - impl HasKey if 'key' ability present
    // =========================================================================
    (@ModuleMembers $($address:literal::)?$module:ident {
        $(#[$meta:meta])*
        struct $Struct:ident$(<$($(!$phantom:ident)? $T:ident$(: $_:ident $(+ $__:ident)*)?),*>)?
        $(has $($ability:ident),+)? {
            $($struct_content:tt)+
        }

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
    //  Empty struct:
    //  - add dummy field
    //  - custom new
    //  - impl Default
    //  - skip ability parsing since empty structs can't have the 'key' ability
    // =========================================================================
    (@ModuleMembers $($address:literal::)?$module:ident {
        $(#[$meta:meta])*
        struct $Struct:ident$(<$(!phantom $T:ident),*>)? $(has $($ability:ident),+)? {}

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
    //  Struct builder
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
    // phantom type parameter
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
    // generic type parameter
    // -------------------------------------------------------------------------
    (@Struct
        $(#[$meta:meta])*
        $Struct:ident$(<$($G:ident),*>)?
        [$T:ident, $($rest:tt)*]
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
