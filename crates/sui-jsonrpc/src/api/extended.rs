// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::proc_macros::rpc;

use crate::msgs::{
    CheckpointedObjectId, EpochInfo, EpochPage, QueryObjectsPage, SuiObjectResponseQuery,
};
use crate::serde::BigInt;

#[rpc(client, namespace = "suix")]
pub trait ExtendedApi {
    /// Return a list of epoch info
    #[method(name = "getEpochs")]
    async fn get_epochs(
        &self,
        cursor: Option<BigInt<u64>>,
        limit: Option<usize>,
        descending_order: Option<bool>,
    ) -> RpcResult<EpochPage>;

    /// Return current epoch info
    #[method(name = "getCurrentEpoch")]
    async fn get_current_epoch(&self) -> RpcResult<EpochInfo>;

    /// Return the list of queried objects. Note that this is an enhanced full node only api.
    #[method(name = "queryObjects")]
    async fn query_objects(
        &self,
        query: SuiObjectResponseQuery,
        cursor: Option<CheckpointedObjectId>,
        limit: Option<usize>,
    ) -> RpcResult<QueryObjectsPage>;

    #[method(name = "getTotalTransactions")]
    async fn get_total_transactions(&self) -> RpcResult<BigInt<u64>>;
}
