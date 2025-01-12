use af_move_type::{FromRawStructError, MoveInstance};
use af_sui_types::StructTag;
use derive_more::{Display, From, IsVariant, TryInto};

#[derive(thiserror::Error, Debug)]
pub enum FromRawEventError {
    #[error(transparent)]
    FromRawStruct(#[from] FromRawStructError),
    #[error("Not an Oracle event name: {0}")]
    Name(String),
}

/// Creates an `$Enum` enum with each `$variant` containing a [`MoveInstance<T>`] where `T` is a
/// type in [`events`](crate::events).
macro_rules! event_instance {
    ($Enum:ident {
        $($variant:ident),+ $(,)?
    }) => {
        /// An AfOracle event instance of any kind.
        // WARN: do not add serde to the below. Since the enum has to remain sorted, adding a
        // variant may change the 'index' of the others, and some serialization formats (e.g., BCS)
        // use the variants' indices; so backwards compatibility could be broken.
        #[remain::sorted]
        #[derive(Clone, Debug, Display, From, IsVariant, TryInto)]
        #[non_exhaustive]
        pub enum $Enum {
            $(
                $variant(MoveInstance<crate::events::$variant>)
            ),+
        }

        impl $Enum {
            pub fn new(type_: StructTag, bcs: impl AsRef<[u8]>) -> Result<Self, FromRawEventError> {
                let name = type_.name.to_string();
                let name_str = name.as_str();
                Ok(match name_str {
                    $(
                        stringify!($variant) => Self::$variant(MoveInstance::from_raw_struct(
                            type_, bcs.as_ref()
                        )?),
                    )+
                    name => return Err(FromRawEventError::Name(name.to_owned())),
                })
            }

            pub fn struct_tag(&self) -> StructTag {
                match self {
                    $(
                        Self::$variant(inner) => inner.type_.clone().into(),
                    )+
                }
            }
        }
    };
}

event_instance!(EventInstance {
    AddedAuthorization,
    CreatedPriceFeed,
    CreatedPriceFeedStorage,
    RemovedAuthorization,
    RemovedPriceFeed,
    UpdatedPriceFeed,
    UpdatedPriceFeedTimeTolerance,
});
