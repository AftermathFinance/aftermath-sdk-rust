// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! An identifier is the name of an entity (module, resource, function, etc) in Move.
//!
//! A valid identifier consists of an ASCII string which satisfies any of the conditions:
//!
//! * The first character is a letter and the remaining characters are letters, digits or
//!   underscores.
//! * The first character is an underscore, and there is at least one further letter, digit or
//!   underscore.
//!
//! The spec for allowed identifiers is similar to Rust's spec
//! ([as of version 1.38](https://doc.rust-lang.org/1.38.0/reference/identifiers.html)).
//!
//! Allowed identifiers are currently restricted to ASCII due to unresolved issues with Unicode
//! normalization. See [Rust issue #55467](https://github.com/rust-lang/rust/issues/55467) and the
//! associated RFC for some discussion. Unicode identifiers may eventually be supported once these
//! issues are worked out.
//!
//! This module only determines allowed identifiers at the bytecode level. Move source code will
//! likely be more restrictive than even this, with a "raw identifier" escape hatch similar to
//! Rust's `r#` identifiers.
//!
//! Among other things, identifiers are used to:
//! * specify keys for lookups in storage
//! * do cross-module lookups while executing transactions

use std::borrow::Borrow;
use std::fmt;

use ref_cast::RefCast;
use sui_sdk_types::Identifier;

// =============================================================================
//  IdentStr
// =============================================================================

/// A borrowed identifier.
///
/// For more details, see the module level documentation.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, RefCast)]
#[repr(transparent)]
pub struct IdentStr(str);

impl IdentStr {
    pub fn new(s: &str) -> Result<&IdentStr, InvalidIdentifierError> {
        if Self::is_valid(s) {
            Ok(IdentStr::ref_cast(s))
        } else {
            Err(InvalidIdentifierError(s.to_owned()))
        }
    }

    /// Compile-time validated constructor from static string slice.
    ///
    /// ### Example
    ///
    /// Creating a valid static or const [`IdentStr`]:
    ///
    /// ```rust
    /// use af_sui_types::IdentStr;
    /// const VALID_IDENT: &'static IdentStr = IdentStr::cast("MyCoolIdentifier");
    ///
    /// const THING_NAME: &'static str = "thing_name";
    /// const THING_IDENT: &'static IdentStr = IdentStr::cast(THING_NAME);
    /// ```
    ///
    /// In contrast, creating an invalid [`IdentStr`] will fail at compile time:
    ///
    /// ```rust,compile_fail
    /// use af_sui_types::IdentStr;
    /// const INVALID_IDENT: &'static IdentStr = IdentStr::cast("123Foo"); // Fails to compile!
    /// ```
    pub const fn cast(s: &'static str) -> &'static IdentStr {
        // Only valid identifier strings are allowed.
        if !is_valid(s) {
            panic!("String is not a valid Move identifier")
        }

        // SAFETY: the following transmute is safe because
        // (1) it's equivalent to the unsafe-reborrow inside IdentStr::ref_cast()
        //     (which we can't use b/c it's not const).
        // (2) we've just asserted that IdentStr impls RefCast<From = str>, which
        //     already guarantees the transmute is safe (RefCast checks that
        //     IdentStr(str) is #[repr(transparent)]).
        // (3) both in and out lifetimes are 'static, so we're not widening the lifetime.
        // (4) we've just asserted that the IdentStr passes the is_valid check.
        unsafe { ::std::mem::transmute::<&'static str, &'static IdentStr>(s) }
    }

    /// Returns true if this string is a valid identifier.
    pub fn is_valid(s: impl AsRef<str>) -> bool {
        is_valid(s.as_ref())
    }

    /// Returns the length of `self` in bytes.
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts `self` to a `&str`.
    ///
    /// This is not implemented as a `From` trait to discourage automatic conversions -- these
    /// conversions should not typically happen.
    pub const fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts `self` to a byte slice.
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Borrow<IdentStr> for Identifier {
    fn borrow(&self) -> &IdentStr {
        let s = self.as_str();
        // SAFETY: same reason as in `IdentStr::cast`
        unsafe { ::std::mem::transmute::<&str, &IdentStr>(s) }
    }
}

impl ToOwned for IdentStr {
    type Owned = Identifier;

    fn to_owned(&self) -> Identifier {
        Identifier::new(&self.0).expect("Identifier validity ensured by IdentStr")
    }
}

impl fmt::Display for IdentStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

// =============================================================================
//  Errors
// =============================================================================

#[derive(thiserror::Error, Debug)]
#[error("Invalid identifier '{0}'")]
pub struct InvalidIdentifierError(pub String);

// =============================================================================
//  Other
// =============================================================================

/// Return true if this character can appear in a Move identifier.
///
/// Note: there are stricter restrictions on whether a character can begin a Move
/// identifier--only alphabetic characters are allowed here.
#[inline]
const fn is_valid_identifier_char(c: char) -> bool {
    matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9')
}

/// Returns `true` if all bytes in `b` after the offset `start_offset` are valid
/// ASCII identifier characters.
const fn all_bytes_valid(b: &[u8], start_offset: usize) -> bool {
    let mut i = start_offset;
    // TODO: use for loop instead of while loop when it's stable in const fn's.
    while i < b.len() {
        if !is_valid_identifier_char(b[i] as char) {
            return false;
        }
        i += 1;
    }
    true
}

/// Describes what identifiers are allowed.
///
/// For now this is deliberately restrictive -- we would like to evolve this in the future.
// TODO: "<SELF>" is coded as an exception. It should be removed once CompiledScript goes away.
const fn is_valid(s: &str) -> bool {
    // Rust const fn's don't currently support slicing or indexing &str's, so we
    // have to operate on the underlying byte slice. This is not a problem as
    // valid identifiers are (currently) ASCII-only.
    let b = s.as_bytes();
    match b {
        b"<SELF>" => true,
        [b'a'..=b'z', ..] | [b'A'..=b'Z', ..] => all_bytes_valid(b, 1),
        [b'_', ..] if b.len() > 1 => all_bytes_valid(b, 1),
        _ => false,
    }
}

// const assert that IdentStr impls RefCast<From = str>
// This assertion is what guarantees the unsafe transmute is safe.
const _: fn() = || {
    const fn assert_impl_all<T: ?Sized + ::ref_cast::RefCast<From = str>>() {}
    assert_impl_all::<IdentStr>();
};

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn with_leading_underscore() {
        let _: Identifier = "_jeet".parse().unwrap();
        let _: Identifier = "_JEET".parse().unwrap();
    }

    /// The same behavior as `sui_types::Identifier` as of `testnet-v1.39.3`.
    #[test]
    fn underscores_only() {
        assert!(Identifier::from_str("_").is_err());
        assert!(Identifier::from_str("__").is_ok());
        assert!(Identifier::from_str("___").is_ok());
        assert!(IdentStr::new("_").is_err());
        assert!(IdentStr::new("__").is_ok());
        assert!(IdentStr::new("___").is_ok());
    }
}
