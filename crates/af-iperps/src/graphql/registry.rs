use af_move_type::MoveInstance;
use af_sui_types::{ObjectId, TypeTag, Version};
use futures::Stream;
use sui_gql_client::GraphQlClient;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::queries::outputs::DynamicField;

use super::Result as QResult;

pub(super) fn query<C>(
    client: &C,
    registry_address: ObjectId,
    version: Option<Version>,
) -> impl Stream<Item = QResult<ObjectId, C>> + '_
where
    C: GraphQlClient,
{
    async_stream::try_stream! {
        let mut has_next_page = true;
        let mut cursor = None;
        while has_next_page {
            let (dfs, cursor_) = client
                .owner_df_contents(registry_address.into(), version, None, cursor)
                .await?;
            cursor = cursor_;
            has_next_page = cursor.is_some();

            for (name, raw) in dfs {
                let DynamicField::Field(_raw) = raw else {
                    continue;
                };
                let TypeTag::Struct(name_type) = name.type_ else {
                    continue;
                };
                if let Ok(key) =
                    MoveInstance::<crate::keys::RegistryMarketInfo>::from_raw_struct(*name_type, &name.bcs)
                {
                    yield key.value.ch_id.bytes;
                }
            }
        }
    }
}
