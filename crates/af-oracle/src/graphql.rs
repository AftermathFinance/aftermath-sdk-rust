use af_move_type::MoveInstance;
use af_sui_types::{Address, ObjectId};
use sui_gql_client::queries::{Error as QueryError, GraphQlClientExt as _};
use sui_gql_client::GraphQlClient;

use crate::oracle::PriceFeed;

type Key = crate::keys::PriceFeedForSource;

/// Error type for [`GraphQlClientExt`].
#[derive(thiserror::Error, Debug)]
pub enum Error<C: std::error::Error> {
    #[error("Querying Owner DF content: {0}")]
    OwnerDfContent(QueryError<C>),

    #[error("BCS De/Ser: {0}")]
    Bcs(#[from] bcs::Error),

    #[error(transparent)]
    FromRawType(#[from] af_move_type::FromRawTypeError),
}

#[trait_variant::make(Send)]
pub trait GraphQlClientExt: GraphQlClient + Sized {
    /// For a `PriceFeedStorage`, get the `PriceFeed` backed by `source_wrapper_id`, if any.
    async fn get_price_feed_for_source(
        &self,
        af_oracle_pkg: Address,
        price_feed_storage: ObjectId,
        source_wrapper_id: ObjectId,
    ) -> Result<Option<MoveInstance<PriceFeed>>, Error<Self::Error>> {
        async move {
            let key = Key::new(source_wrapper_id.into()).move_instance(af_oracle_pkg);
            let raw_move_value = self
                .owner_df_content(price_feed_storage.into(), key.try_into()?, None)
                .await;
            match raw_move_value {
                Ok(raw) => Ok(Some(raw.try_into()?)),
                Err(QueryError::MissingData(_)) => Ok(None),
                Err(err) => Err(Error::OwnerDfContent(err)),
            }
        }
    }
}

impl<T: GraphQlClient> GraphQlClientExt for T {}
