use af_move_type::MoveInstance;
use af_sui_types::Address;
use sui_framework_sdk::object::ID;
use sui_gql_client::GraphQlClient;
use sui_gql_client::queries::model::outputs::DynamicField;
use sui_gql_client::queries::{Error as QueryError, GraphQlClientExt as _};

type Key = crate::wrapper::PythPriceInfoId;

/// Error for [`GraphQlClientExt`].
#[derive(thiserror::Error, Debug)]
pub enum Error<C: std::error::Error> {
    #[error("Querying Owner DF content: {0}")]
    ObjectDfQuery(QueryError<C>),

    #[error("BCS De/Ser: {0}")]
    Bcs(#[from] sui_sdk_types::bcs::Error),

    #[error(transparent)]
    FromRawType(#[from] af_move_type::FromRawTypeError),

    #[error("Found a struct object, expecting Address")]
    RawInvalidType,
}

pub async fn query<C>(
    client: &C,
    pyth_wrapper_pkg: Address,
    price_feed: Address,
) -> Result<Address, Error<C::Error>>
where
    C: GraphQlClient,
{
    let key = Key::new().move_instance(pyth_wrapper_pkg);
    let raw_move_value = client
        .object_df_by_name(price_feed, key.try_into()?, None)
        .await;
    let move_id: MoveInstance<ID> = match raw_move_value {
        Ok(DynamicField::Field(raw)) => raw.try_into()?,
        Ok(DynamicField::Object(_, _)) => return Err(Error::RawInvalidType),
        Err(err) => return Err(Error::ObjectDfQuery(err)),
    };

    Ok(move_id.value.bytes)
}
