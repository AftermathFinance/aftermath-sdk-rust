// use af_move_type::MoveInstance;
// use af_sui_types::{Address, TypeTag, Version};
// use futures::{Stream, StreamExt};
// use sui_gql_client::GraphQlClient;
// use sui_gql_client::queries::GraphQlClientExt as _;
// use sui_gql_client::queries::outputs::DynamicField;

// use super::Result as QResult;

// pub(super) async fn query<C>(
//     client: &C,
//     registry_address: Address,
//     version: Option<Version>,
// ) -> Result<impl Stream<Item = QResult<Address, C>> + '_, C::Error>
// where
//     C: GraphQlClient,
// {
//     let dfs = std::pin::pin!(client.object_dfs(registry_address, version, None).await);
//     let res = vec

//     while let Some(df) = dfs.next().await {
//         let (name, raw) = df?;

//         let DynamicField::Field(_raw) = raw else {
//             continue;
//         };
//         let TypeTag::Struct(name_type) = name.type_ else {
//             continue;
//         };
//         if let Ok(key) =
//             MoveInstance::<crate::keys::RegistryMarketInfo>::from_raw_struct(*name_type, &name.bcs)
//         {
//             yield key.value.ch_id.bytes;
//         }
//     }
//     // async_stream::try_stream! {
//     //     let mut has_next_page = true;
//     //     let mut cursor = None;
//     //     while has_next_page {
//     //         let (dfs, cursor_) = client
//     //             .owner_df_contents(registry_address, version, None, cursor)
//     //             .await?;
//     //         cursor = cursor_;
//     //         has_next_page = cursor.is_some();

//     //         for (name, raw) in dfs {
//     //             let DynamicField::Field(_raw) = raw else {
//     //                 continue;
//     //             };
//     //             let TypeTag::Struct(name_type) = name.type_ else {
//     //                 continue;
//     //             };
//     //             if let Ok(key) =
//     //                 MoveInstance::<crate::keys::RegistryMarketInfo>::from_raw_struct(*name_type, &name.bcs)
//     //             {
//     //                 yield key.value.ch_id.bytes;
//     //             }
//     //         }
//     //     }
//     // }
// }
