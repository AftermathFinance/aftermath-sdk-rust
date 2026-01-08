#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

//! Builder for programmable transactions.
//!
//! Check out the [`ptb`](crate::ptb) and [`ptbuilder`](crate::ptbuilder) macros for an ergonomic
//! way of building transactions, or
//! [`ProgrammableTransactionBuilder`](crate::ProgrammableTransactionBuilder) for a macro-less
//! approach.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
#[doc(no_inline)]
pub use sui_sdk_types::Address;
#[doc(no_inline)]
pub use sui_sdk_types::Argument;
#[doc(hidden)]
pub use sui_sdk_types::Identifier;
#[doc(inline)]
pub use sui_sdk_types::Input;
#[doc(no_inline)]
pub use sui_sdk_types::MoveCall;
#[doc(no_inline)]
pub use sui_sdk_types::TypeTag;
use sui_sdk_types::bcs::ToBcs;
use sui_sdk_types::{Mutability, ProgrammableTransaction, SharedInput};

#[cfg(test)]
mod tests;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Serializing to BCS: {0}")]
    Bcs(#[from] sui_sdk_types::bcs::Error),

    #[error("invariant violation! object has pure argument")]
    ObjInvariantViolation,

    #[error("invariant violation! object has id does not match call arg")]
    InvalidObjArgUpdate,

    #[error(transparent)]
    MismatchedObjArgKinds(Box<MismatchedObjArgKindsError>),

    #[error("tried to use a pure or funds withdrawal argument as an object argument")]
    NotAnObjectArg,
}

#[derive(thiserror::Error, Debug)]
#[error(
    "Mismatched Object argument kind for object {id}. \
        {old_value:?} is not compatible with {new_value:?}"
)]
pub struct MismatchedObjArgKindsError {
    pub id: Address,
    pub old_value: Input,
    pub new_value: Input,
}

/// Builder for a [`ProgrammableTransaction`].
#[derive(Clone, Debug, Default)]
pub struct ProgrammableTransactionBuilder {
    inputs: IndexMap<BuilderArg, Input>,
    commands: Vec<sui_sdk_types::Command>,
}

/// Base API.
impl ProgrammableTransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> ProgrammableTransaction {
        let Self { inputs, commands } = self;
        let inputs = inputs.into_values().collect();
        ProgrammableTransaction { inputs, commands }
    }

    /// Potentially adds a pure argument to the PTB.
    ///
    /// May not create a new PTB input if a previous one already has the same contents.
    pub fn pure<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<Argument> {
        Ok(self.pure_bytes(value.to_bcs()?, false))
    }

    /// Like [`Self::pure`] but forces a separate input entry
    pub fn force_separate_pure<T: Serialize>(&mut self, value: T) -> Result<Argument> {
        Ok(self.pure_bytes(value.to_bcs()?, true))
    }

    /// Adds a pure argument to the PTB.
    ///
    /// # Arguments
    /// - `bytes`: the BCS-serialized contents of the argument
    /// - `force_separate`: whether to force a separate input argument to the PTB, else the builder
    ///   re-uses a previously declared input argument if it has the same contents.
    pub fn pure_bytes(&mut self, bytes: Vec<u8>, force_separate: bool) -> Argument {
        let key = if force_separate {
            BuilderArg::ForcedNonUniquePure(self.inputs.len())
        } else {
            BuilderArg::Pure(bytes.clone())
        };
        let (i, _) = self.inputs.insert_full(key, Input::Pure(bytes));
        Argument::Input(i as u16)
    }

    /// Adds an object input to the PTB, returning the corresponding argument which can be used in
    /// the body.
    ///
    /// May fail if overriding a previously declared input.
    pub fn obj<O: Into<Input>>(&mut self, obj_arg: O) -> Result<Argument> {
        let obj_arg = obj_arg.into();
        let id = match &obj_arg {
            Input::Pure { .. } => return Err(Error::NotAnObjectArg),
            Input::ImmutableOrOwned(object_reference) => *object_reference.object_id(),
            Input::Shared(shared) => shared.object_id(),
            Input::Receiving(object_reference) => *object_reference.object_id(),
            Input::FundsWithdrawal(_) => return Err(Error::NotAnObjectArg),
            _ => panic!("unknown Input variant"),
        };
        let key = BuilderArg::Object(id);
        let mut input_arg = obj_arg;

        if let Some(old_value) = self.inputs.get(&key) {
            // Check if the key hash didn't collide with a previous pure input
            if matches!(old_value, Input::Pure { .. }) {
                return Err(Error::ObjInvariantViolation);
            }

            input_arg = match (old_value, input_arg) {
                // The only update allowed: changing the `mutable` flag for a shared object input
                (Input::Shared(shared1), Input::Shared(shared2))
                    if shared1.version() == shared2.version() =>
                {
                    if shared1.object_id() != shared2.object_id() {
                        return Err(Error::InvalidObjArgUpdate);
                    }
                    let mutable =
                        shared1.mutability().is_mutable() || shared2.mutability().is_mutable();
                    Input::Shared(SharedInput::new(
                        shared2.object_id(),
                        shared2.version(),
                        Mutability::from(mutable),
                    ))
                }

                // Changing anything else about an existing object input is disallowed
                (old_value, new_value) if old_value != &new_value => {
                    return Err(Error::MismatchedObjArgKinds(Box::new(
                        MismatchedObjArgKindsError {
                            id,
                            old_value: old_value.clone(),
                            new_value,
                        },
                    )));
                }

                // If we already declared this exact same object input in the transaction, it will
                // be automatically reused
                (_, new_value) => new_value,
            };
        }

        let (i, _) = self.inputs.insert_full(key, input_arg);
        Ok(Argument::Input(i as u16))
    }

    /// Add a command to the PTB.
    ///
    /// This will come after any commands that were previously declared.
    pub fn command(&mut self, command: impl Into<sui_sdk_types::Command>) -> Argument {
        let i = self.commands.len();
        self.commands.push(command.into());
        Argument::Result(i as u16)
    }
}

/// Extensions to the base API.
impl ProgrammableTransactionBuilder {
    /// Like `.command(Command::SplitCoins(coin_arg, balances))`, but also takes care of unpacking
    /// each entry in the returned vector as its own [`Argument`].
    ///
    /// # Panics
    ///
    /// Panics if the `balances` input vector has a length that exceeds [`u16::MAX`].
    pub fn split_coins_into_vec(
        &mut self,
        coin: Argument,
        amounts: Vec<Argument>,
    ) -> Vec<Argument> {
        let idxs = 0..amounts.len() as u16;
        let Argument::Result(coin_vec) = self.command(Command::SplitCoins(coin, amounts)) else {
            panic!("ProgrammableTransactionBuilder::command always gives an Argument::Result")
        };
        idxs.map(|i| Argument::NestedResult(coin_vec, i)).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum BuilderArg {
    Object(Address),
    Pure(Vec<u8>),
    ForcedNonUniquePure(usize),
}

impl From<ProgrammableTransactionBuilder> for ProgrammableTransaction {
    fn from(value: ProgrammableTransactionBuilder) -> Self {
        value.finish()
    }
}

impl TryFrom<ProgrammableTransaction> for ProgrammableTransactionBuilder {
    type Error = Error;

    fn try_from(
        ProgrammableTransaction { inputs, commands }: ProgrammableTransaction,
    ) -> Result<Self> {
        use Input::*;
        let mut self_ = Self::new();
        for input in inputs {
            match input {
                Pure(value) => {
                    self_.pure_bytes(value, true);
                }
                ImmutableOrOwned(object_reference) => {
                    self_.obj(Input::ImmutableOrOwned(object_reference))?;
                }
                Shared(shared) => {
                    self_.obj(Input::Shared(shared.clone()))?;
                }
                Receiving(object_reference) => {
                    self_.obj(Input::Receiving(object_reference))?;
                }
                FundsWithdrawal(funds_withdrawal) => {
                    let key = BuilderArg::ForcedNonUniquePure(self_.inputs.len());
                    self_
                        .inputs
                        .insert_full(key, Input::FundsWithdrawal(funds_withdrawal));
                }
                _ => panic!("unknown Input variant"),
            }
        }
        for command in commands {
            self_.command(command);
        }
        Ok(self_)
    }
}

// =============================================================================
//  Command compat for migration
// =============================================================================

/// A single command in a programmable transaction.
///
/// This type is here for backwards compatibility purposes, as [`sui_sdk_types::Command`]
/// has a different shape that would be incompatible with the [`ptb!`] syntax.
///
/// The actual resulting [`ProgrammableTransaction`] does not contain this type.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Command {
    /// A call to either an entry or a public Move function.
    ///
    /// Either an entry function or a public function (which cannot return references).
    MoveCall(Box<MoveCall>),
    /// `(Vec<forall T:key+store. T>, address)`
    /// It sends n-objects to the specified address. These objects must have store
    /// (public transfer) and either the previous owner must be an address or the object must
    /// be newly created.
    TransferObjects(Vec<Argument>, Argument),
    /// `(&mut Coin<T>, Vec<u64>)` -> `Vec<Coin<T>>`
    /// It splits off some amounts into a new coins with those amounts
    SplitCoins(Argument, Vec<Argument>),
    /// `(&mut Coin<T>, Vec<Coin<T>>)`
    /// It merges n-coins into the first coin
    MergeCoins(Argument, Vec<Argument>),
    /// Publishes a Move package. It takes the package bytes and a list of the package's transitive
    /// dependencies to link against on-chain.
    Publish(Vec<Vec<u8>>, Vec<Address>),
    /// `forall T: Vec<T> -> vector<T>`
    /// Given n-values of the same type, it constructs a vector. For non objects or an empty vector,
    /// the type tag must be specified.
    MakeMoveVec(Option<TypeTag>, Vec<Argument>),
    /// Upgrades a Move package
    /// Takes (in order):
    /// 1. A vector of serialized modules for the package.
    /// 2. A vector of object ids for the transitive dependencies of the new package.
    /// 3. The object ID of the package being upgraded.
    /// 4. An argument holding the `UpgradeTicket` that must have been produced from an earlier command in the same
    ///    programmable transaction.
    Upgrade(Vec<Vec<u8>>, Vec<Address>, Address, Argument),
}

#[allow(clippy::fallible_impl_from)]
impl From<sui_sdk_types::Command> for Command {
    fn from(value: sui_sdk_types::Command) -> Self {
        use sui_sdk_types::Command::*;
        match value {
            MoveCall(args) => Self::MoveCall(Box::new(args)),
            TransferObjects(args) => Self::TransferObjects(args.objects, args.address),
            SplitCoins(args) => Self::SplitCoins(args.coin, args.amounts),
            MergeCoins(args) => Self::MergeCoins(args.coin, args.coins_to_merge),
            Publish(args) => Self::Publish(args.modules, args.dependencies),
            MakeMoveVector(args) => Self::MakeMoveVec(args.type_, args.elements),
            Upgrade(args) => {
                Self::Upgrade(args.modules, args.dependencies, args.package, args.ticket)
            }
            _ => panic!("unknown Command variant"),
        }
    }
}

impl From<Command> for sui_sdk_types::Command {
    fn from(value: Command) -> Self {
        use Command::*;
        use sui_sdk_types::{
            MakeMoveVector,
            MergeCoins,
            Publish,
            SplitCoins,
            TransferObjects,
            Upgrade,
        };
        match value {
            MoveCall(move_call) => Self::MoveCall(*move_call),
            TransferObjects(objects, address) => {
                Self::TransferObjects(TransferObjects { objects, address })
            }
            SplitCoins(coin, amounts) => Self::SplitCoins(SplitCoins { coin, amounts }),
            MergeCoins(coin, coins_to_merge) => Self::MergeCoins(MergeCoins {
                coin,
                coins_to_merge,
            }),
            Publish(modules, dependencies) => Self::Publish(Publish {
                modules,
                dependencies,
            }),
            MakeMoveVec(type_, elements) => {
                Self::MakeMoveVector(MakeMoveVector { type_, elements })
            }
            Upgrade(modules, dependencies, package, ticket) => Self::Upgrade(Upgrade {
                modules,
                dependencies,
                package,
                ticket,
            }),
        }
    }
}

impl Command {
    pub fn move_call(
        package: Address,
        module: Identifier,
        function: Identifier,
        type_arguments: Vec<TypeTag>,
        arguments: Vec<Argument>,
    ) -> Self {
        Self::MoveCall(Box::new(MoveCall {
            package,
            module,
            function,
            type_arguments,
            arguments,
        }))
    }

    pub const fn make_move_vec(ty: Option<TypeTag>, args: Vec<Argument>) -> Self {
        Self::MakeMoveVec(ty, args)
    }
}

// =============================================================================
//  Macro helper
// =============================================================================

/// Build a programmable transaction using Move-like syntax.
///
/// # Overview
///
/// This automatically creates and finishes a [`ProgrammableTransactionBuilder`] and allows users
/// to declare:
/// - packages the transaction uses
/// - type arguments for functions
/// - object/pure inputs for the transaction
/// - Move calls
/// - Built-in PTB commands
///
/// Every Move call and built-in PTB command declared withing the macro's scope can be thought of
/// as happening in 'programmable transaction time'. In this way, the macro also helps users more
/// clearly separate what's being executed at Rust's runtime and chain's runtime (once the
/// transaction is execute by validators).
///
/// ## Packages
///
/// Move functions expect the [`Address`] of their package in the transaction payload (see
/// [`MoveCall`]). One can declare the packages using the syntax
/// ```no_run
/// # use sui_sdk_types::Address;
/// let package_name = Address::new(rand::random());
/// let object_id = Address::new(rand::random());
/// af_ptbuilder::ptb!(
///     package package_name;
///     package package_name: object_id;
/// // ...
/// );
/// ```
/// Similar to struct initialization syntax;
///
/// ## Type arguments
///
/// Move functions that have type arguments expect [`TypeTag`] arguments in the transaction payload
/// (see [`MoveCall`]). One can declare these variables using the syntax
/// ```no_run
/// # use sui_sdk_types::TypeTag;
/// let T = TypeTag::U8;
/// let type_tag = TypeTag::U32;
/// af_ptbuilder::ptb!(
///     type T;
///     type T = type_tag;
/// // ...
/// );
/// ```
///
/// ## Object/Pure inputs
///
/// [`ProgrammableTransaction`]s need all their inputs declared upfront. One can
/// declare the two types of inputs using the syntax
/// ```no_run
/// # use sui_sdk_types::{Address, Input, Mutability, SharedInput};
/// let clock = Input::Shared(SharedInput::new(
///     Address::from_static("0x6"),
///     1,
///     Mutability::from(false),
/// ));
/// let object = Input::Shared(SharedInput::new(
///     Address::new(rand::random()),
///     1,
///     Mutability::from(true),
/// ));
/// let count = &0_u64;
/// af_ptbuilder::ptb!(
///     input obj clock;
///     input obj another: object;
///     input pure count;
///     input pure more: &1_u32;
///     // ...
/// );
/// # eyre::Ok(())
/// ```
/// Similar to struct initialization syntax. `input obj`s expect [`Input`] values and
/// become object [`Input`]s in the transaction payload. `input pure`s expect any type `T` that
/// is [`Serialize`] `+ ?Sized` (see [`ProgrammableTransactionBuilder::pure`] for the internals) and
/// become [`Input::Pure`]s in the transaction payload. Within the macro scope, both variables
/// are [`Argument::Input`]s and can be used in Move/built-in calls.
///
/// ## Move calls
///
/// Use the syntax
/// ```no_run
/// # af_ptbuilder::ptb!(
/// # package package: sui_sdk_types::Address::new(rand::random());
/// # type T = sui_sdk_types::TypeTag::U8;
/// # input pure arg: &0_u32;
///     package::module::function<T>(arg);
/// # );
/// # eyre::Ok(())
/// ````
/// To include a [`MoveCall`] in the transaction payload. `package`,`T`, and `arg`
/// must have been declared earlier. `module` and `function` are simply pure identifiers[^1]. One
/// can of course declare more than one type argument if the function requires, or none if the
/// function does not have type parameters.
///
/// Functions that return can have their results assigned to a value or unpacked into several ones:
/// ```no_run
/// # use sui_sdk_types::{Address, Input, SharedInput, Mutability};
/// # let clock = Input::Shared(SharedInput::new(
/// #     Address::from_static("0x6"),
/// #     1,
/// #     Mutability::from(false),
/// # ));
/// # let clock2 = clock.clone();
/// # let clock3 = clock.clone();
/// # af_ptbuilder::ptb!(
/// # package package: Address::new(rand::random());
/// # input obj a: clock;
/// # input obj b: clock2;
/// # input obj arg: clock3;
/// let result = package::module::function(a, b);
/// let (a, b) = package::module::function(arg);
/// # );
/// # eyre::Ok(())
/// ```
/// These, of course, happen at 'programmable transaction time' and the result are
/// [`Argument::Result`]s that can be passed to other functions.
///
/// ## Built-in commands
///
/// Sui PTBs have access to some calls that do not declare a package, module and function. These
/// use the syntax:
/// ```text
/// command! Variant(x, y, ...);
/// ```
/// The result of the command can be optionally assigned or unpacked (`let a =` or
/// `let (a, b) =`). `Variant` refers to the variant of [`Command`] to use. See its
/// documentation for more information.
///
/// # Example
///
/// ```no_run
/// use af_ptbuilder::ptb;
/// use sui_sdk_types::{Address, Input, Mutability, SharedInput, TypeTag};
///
/// let foo = Address::from_static("0xbeef");
/// let otw: TypeTag = "0x2::sui::SUI".parse()?;
/// let registry = Input::Shared(SharedInput::new(
///     Address::from_static("0xdeed"),
///     1,
///     Mutability::from(true),
/// ));
/// let sender = Address::from_static("0xabcd");
///
/// ptb!(
///     package foo;
///
///     type T = otw;
///
///     input obj registry;
///     input pure sender: &sender;
///
///     let account = foo::registry::create_account<T>(registry);
///     command! TransferObjects(vec![account], sender);
/// );
/// # eyre::Ok(())
/// ```
///
/// [^1]: [`Identifier`]
#[macro_export]
macro_rules! ptb {
    ($($tt:tt)*) => {
        {
            let mut builder = $crate::ProgrammableTransactionBuilder::new();
            $crate::ptbuilder!(builder { $($tt)* });
            builder.finish()
        }
    };
}

/// Build a programmable transaction using Move-like syntax and an existing builder.
///
/// This will make the package, type, input and argument variables declared inside the macro
/// available in the outer scope.
///
/// # Overview
///
/// This allows users to incrementally build a programmable transaction using an existing
/// [`ProgrammableTransactionBuilder`] with the syntax
/// ```no_run
/// # use af_ptbuilder::ProgrammableTransactionBuilder;
/// let mut builder = ProgrammableTransactionBuilder::new();
/// af_ptbuilder::ptbuilder!(builder {
///     // ...
/// });
/// ```
/// where everything inside the braces uses the same syntax as [`ptb!`]. The user is responsible
/// for initializing the builder and calling [`ProgrammableTransactionBuilder::finish`] at the end.
///
/// This can be useful if the number of calls is only known at runtime or if it's desirable to only
/// include some calls based on some runtime logic. It still allows users to use a convenient
/// syntax and separate what happens at 'programmable transaction time'.
#[macro_export]
macro_rules! ptbuilder {
    ($builder:ident {}) => { };

    ($builder:ident {
        package $name:ident $value:literal;
        $($tt:tt)*
    }) => {
        let $name: $crate::Address = $value.parse()?;

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        package $name:ident;
        $($tt:tt)*
    }) => {
        let $name: $crate::Address = $name;

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        package $name:ident: $value:expr_2021;
        $($tt:tt)*
    }) => {
        let $name: $crate::Address = $value;

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        input pure $name:ident;
        $($tt:tt)*
    }) => {
        let $name = $builder.pure($name)?;

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        input pure $name:ident: $value:expr_2021;
        $($tt:tt)*
    }) => {
        let $name = $builder.pure($value)?;

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        input obj $name:ident;
        $($tt:tt)*
    }) => {
        let $name = $builder.obj($name)?;

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        input obj $name:ident: $value:expr_2021;
        $($tt:tt)*
    }) => {
        let $name = $builder.obj($value)?;

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        type $T:ident;
        $($tt:tt)*
    }) => {
        #[allow(non_snake_case)]
        let $T: $crate::TypeTag = $T;

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        type $T:ident = $value:expr_2021;
        $($tt:tt)*
    }) => {
        #[allow(non_snake_case)]
        let $T: $crate::TypeTag = $value;

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        $package:ident::$module:ident::$fun:ident$(<$($T:ident),+>)?($($arg:ident),* $(,)?);
        $($tt:tt)*
    }) => {
        let _module = stringify!($module);
        let _fun = stringify!($fun);
        $builder.command($crate::Command::move_call(
            $package,
            $crate::Identifier::from_static(_module),
            $crate::Identifier::from_static(_fun),
            vec![$($($T.clone()),+)?],
            vec![$($arg),*]
        ));

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        let $ret:ident = $package:ident::$module:ident::$fun:ident$(<$($T:ident),+>)?($($arg:ident),* $(,)?);
        $($tt:tt)*
    }) => {
        let _module = stringify!($module);
        let _fun = stringify!($fun);
        let $ret = $builder.command($crate::Command::move_call(
            $package,
            $crate::Identifier::from_static(_module),
            $crate::Identifier::from_static(_fun),
            vec![$($($T.clone()),+)?],
            vec![$($arg),*]
        ));

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        let ($($ret:ident),+) = $package:ident::$module:ident::$fun:ident$(<$($T:ident),+>)?($($arg:ident),* $(,)?);
        $($tt:tt)*
    }) => {
        let _module = stringify!($module);
        let _fun = stringify!($fun);
        let rets = $builder.command($crate::Command::move_call(
            $package,
            $crate::Identifier::from_static(_module),
            $crate::Identifier::from_static(_fun),
            vec![$($($T.clone()),+)?],
            vec![$($arg),*]
        ));
        $crate::unpack_arg!(rets => { $($ret),+ });

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        $(let $ret:ident =)? command! $variant:ident($($args:tt)*);
        $($tt:tt)*
    }) => {
        $(let $ret =)? $builder.command($crate::Command::$variant($($args)*));

        $crate::ptbuilder!($builder { $($tt)* });
    };

    ($builder:ident {
        let ($($ret:ident),+) = command! $variant:ident($($args:tt)*);
        $($tt:tt)*
    }) => {
        let rets = $builder.command($crate::Command::$variant($($args)*));
        $crate::unpack_arg!(rets => { $($ret),+ });

        $crate::ptbuilder!($builder { $($tt)* });
    };
}

/// Unpack the result of a programmable transaction call.
///
/// Useful for unpacking results from functions that return tuple or vector types.
///
/// # Example
/// ```
/// use af_ptbuilder::ProgrammableTransactionBuilder;
/// use sui_sdk_types::Argument;
///
/// let mut builder = ProgrammableTransactionBuilder::new();
/// let arg = Argument::Result(0);
/// af_ptbuilder::unpack_arg!(arg => { sub1, sub2 });
/// ```
#[macro_export]
macro_rules! unpack_arg {
    ($arg:expr_2021 => {
        $($name:ident),+ $(,)?
    }) => {
        let ($($name),+) = if let $crate::Argument::Result(tuple) = $arg {
            let mut index = 0;
            $(
                let $name = $crate::Argument::NestedResult(
                    tuple, index
                );
                index += 1;
            )+
            ($($name),+)
        } else {
            panic!(
                "ProgrammableTransactionBuilder::command should always give a Argument::Result"
            )
        };
    };
}
