// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use af_sui_types::{Address as SuiAddress, ObjectId, TransactionDigest};
use jsonrpsee::proc_macros::rpc;

use crate::msgs::{
    DynamicFieldName,
    DynamicFieldPage,
    EventFilter,
    EventID,
    EventPage,
    ObjectsPage,
    Page,
    SuiEvent,
    SuiObjectResponse,
    SuiObjectResponseQuery,
    SuiTransactionBlockEffects,
    SuiTransactionBlockResponseQuery,
    TransactionBlocksPage,
    TransactionFilter,
};

#[rpc(client, namespace = "suix")]
pub trait IndexerApi {
    /// Return the list of objects owned by an address.
    /// Note that if the address owns more than `QUERY_MAX_RESULT_LIMIT` objects,
    /// the pagination is not accurate, because previous page may have been updated when
    /// the next page is fetched.
    /// Please use suix_queryObjects if this is a concern.
    #[method(name = "getOwnedObjects")]
    async fn get_owned_objects(
        &self,
        address: SuiAddress,
        query: Option<SuiObjectResponseQuery>,
        cursor: Option<ObjectId>,
        limit: Option<usize>,
    ) -> RpcResult<ObjectsPage>;

    /// Return list of transactions for a specified query criteria.
    #[method(name = "queryTransactionBlocks")]
    async fn query_transaction_blocks(
        &self,
        query: SuiTransactionBlockResponseQuery,
        cursor: Option<TransactionDigest>,
        limit: Option<usize>,
        descending_order: Option<bool>,
    ) -> RpcResult<TransactionBlocksPage>;

    /// Return list of events for a specified query criteria.
    #[method(name = "queryEvents")]
    async fn query_events(
        &self,
        query: EventFilter,
        cursor: Option<EventID>,
        limit: Option<usize>,
        descending_order: Option<bool>,
    ) -> RpcResult<EventPage>;

    /// Subscribe to a stream of Sui event
    #[subscription(name = "subscribeEvent", item = SuiEvent)]
    fn subscribe_event(&self, filter: EventFilter);

    /// Subscribe to a stream of Sui transaction effects
    #[subscription(name = "subscribeTransaction", item = SuiTransactionBlockEffects)]
    fn subscribe_transaction(&self, filter: TransactionFilter);

    /// Return the list of dynamic field objects owned by an object.
    #[method(name = "getDynamicFields")]
    async fn get_dynamic_fields(
        &self,
        parent_object_id: ObjectId,
        cursor: Option<ObjectId>,
        limit: Option<usize>,
    ) -> RpcResult<DynamicFieldPage>;

    /// Return the dynamic field object information for a specified object
    #[method(name = "getDynamicFieldObject")]
    async fn get_dynamic_field_object(
        &self,
        parent_object_id: ObjectId,
        name: DynamicFieldName,
    ) -> RpcResult<SuiObjectResponse>;

    /// Return the resolved address given resolver and name
    #[method(name = "resolveNameServiceAddress")]
    async fn resolve_name_service_address(&self, name: String) -> RpcResult<Option<SuiAddress>>;

    /// Return the resolved names given address,
    /// if multiple names are resolved, the first one is the primary name.
    #[method(name = "resolveNameServiceNames")]
    async fn resolve_name_service_names(
        &self,
        address: SuiAddress,
        cursor: Option<ObjectId>,
        limit: Option<usize>,
    ) -> RpcResult<Page<String, ObjectId>>;
}
