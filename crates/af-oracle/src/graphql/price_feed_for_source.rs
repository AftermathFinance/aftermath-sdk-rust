use af_move_type::{FromRawStructError, MoveInstance};
use af_sui_types::Address;
use sui_gql_client::GraphQlClient;
use sui_gql_client::queries::model::outputs::DynamicField;
use sui_gql_client::queries::{Error as QueryError, GraphQlClientExt as _};

use crate::oracle::PriceFeed;

type Key = crate::keys::PriceFeedForSource;

pub(crate) async fn query<C>(
    client: &C,
    af_oracle_pkg: Address,
    price_feed_storage: Address,
    source_wrapper_id: Address,
) -> Result<Option<MoveInstance<PriceFeed>>, Error<C::Error>>
where
    C: GraphQlClient,
{
    let key = Key::new(source_wrapper_id.into()).move_instance(af_oracle_pkg);
    let raw_move_value = client
        .object_df_by_name(price_feed_storage, key.try_into()?, None)
        .await;
    match raw_move_value {
        Ok(DynamicField::Field(raw)) => Ok(Some(raw.try_into()?)),
        Ok(DynamicField::Object(_key, raw)) => Ok(Some(raw.try_into()?)),
        Err(QueryError::MissingData(_)) => Ok(None),
        Err(err) => Err(Error::OwnerDfContent(err)),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error<C: std::error::Error> {
    #[error("Querying Owner DF content: {0}")]
    OwnerDfContent(QueryError<C>),

    #[error("BCS De/Ser: {0}")]
    Bcs(#[from] sui_sdk_types::bcs::Error),

    #[error(transparent)]
    FromRawType(#[from] af_move_type::FromRawTypeError),

    #[error(transparent)]
    FromRawStruct(#[from] FromRawStructError),
}
