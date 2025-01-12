pub mod balance9;
pub mod errors;
pub mod fixed;
pub mod i256;
pub mod ifixed;
pub mod onchain;

pub use balance9::Balance9;
pub use fixed::*;
pub use i256::*;
pub use ifixed::*;

macro_rules! reuse_op_for_assign {
    ($type:ty {
        $($Assign:ident $method:ident $op:tt),* $(,)?
    }) => {
        $(
            impl $Assign for $type {
                fn $method(&mut self, rhs: Self) {
                    *self = *self $op rhs
                }
            }
        )*
    };
}

pub(crate) use reuse_op_for_assign;
