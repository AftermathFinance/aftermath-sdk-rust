//! One-time-witness type.

use serde::{Deserialize, Serialize};

use crate::MoveStruct;

/// Generic type signaling a one-time-witness type argument.
///
/// None of address, module and name are known at compile time, only that there are no type
/// parameters
///
/// This is useful when calling associated methods on a move type that don't depend on the specific
/// type argument.
///
/// # Examples
/// ```
/// use af_move_type::{MoveStruct, MoveType, otw::Otw};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(MoveStruct, Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
/// #[move_(address = "0x2", module = balance)]
/// pub struct Balance<T: MoveType> {
///     value: u64,
///     _otw: std::marker::PhantomData<T>,
/// }
///
/// // for compilation purposes
/// impl<T: MoveType> std::fmt::Display for Balance<T> {
/// fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///     todo!()
/// }
/// }
///
/// let address = "0x2".parse().unwrap();
/// let module = "sui".parse().unwrap();
/// let name = "SUI".parse().unwrap();
/// let otw = Otw::type_(address, module, name);
/// let usdc_type = Balance::<Otw>::type_(otw);
/// ```
#[derive(MoveStruct, Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[move_(crate = crate)]
#[move_(nameless)]
pub struct Otw {
    dummy_field: bool,
}

impl Otw {
    pub fn new() -> Self {
        Default::default()
    }
}

// This should never be used since OTWs are never instantiated.
impl std::fmt::Display for Otw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OTW")
    }
}
