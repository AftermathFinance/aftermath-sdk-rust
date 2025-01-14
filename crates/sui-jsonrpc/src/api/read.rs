// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use af_sui_types::{ObjectId, TransactionDigest};
use jsonrpsee::proc_macros::rpc;
use sui_sdk_types::types::Version;

use crate::msgs::{
    Checkpoint,
    CheckpointId,
    CheckpointPage,
    ProtocolConfigResponse,
    SuiEvent,
    SuiGetPastObjectRequest,
    SuiObjectDataOptions,
    SuiObjectResponse,
    SuiPastObjectResponse,
    SuiTransactionBlockResponse,
    SuiTransactionBlockResponseOptions,
};
use crate::serde::BigInt;

#[rpc(client, namespace = "sui")]
pub trait ReadApi {
    /// Return the transaction response object.
    #[method(name = "getTransactionBlock")]
    async fn get_transaction_block(
        &self,
        digest: TransactionDigest,
        options: Option<SuiTransactionBlockResponseOptions>,
    ) -> RpcResult<SuiTransactionBlockResponse>;

    /// Returns an ordered list of transaction responses
    /// The method will throw an error if the input contains any duplicate or
    /// the input size exceeds QUERY_MAX_RESULT_LIMIT
    #[method(name = "multiGetTransactionBlocks")]
    async fn multi_get_transaction_blocks(
        &self,
        digests: Vec<TransactionDigest>,
        options: Option<SuiTransactionBlockResponseOptions>,
    ) -> RpcResult<Vec<SuiTransactionBlockResponse>>;

    /// Return the object information for a specified object
    #[method(name = "getObject")]
    async fn get_object(
        &self,
        object_id: ObjectId,
        options: Option<SuiObjectDataOptions>,
    ) -> RpcResult<SuiObjectResponse>;

    /// Return the object data for a list of objects
    #[method(name = "multiGetObjects")]
    async fn multi_get_objects(
        &self,
        object_ids: Vec<ObjectId>,
        options: Option<SuiObjectDataOptions>,
    ) -> RpcResult<Vec<SuiObjectResponse>>;

    /// Note there is no software-level guarantee/SLA that objects with past versions
    /// can be retrieved by this API, even if the object and version exists/existed.
    /// The result may vary across nodes depending on their pruning policies.
    /// Return the object information for a specified version
    #[method(name = "tryGetPastObject")]
    async fn try_get_past_object(
        &self,
        object_id: ObjectId,
        version: Version,
        options: Option<SuiObjectDataOptions>,
    ) -> RpcResult<SuiPastObjectResponse>;

    /// Note there is no software-level guarantee/SLA that objects with past versions
    /// can be retrieved by this API, even if the object and version exists/existed.
    /// The result may vary across nodes depending on their pruning policies.
    /// Return the object information for a specified version
    #[method(name = "tryMultiGetPastObjects")]
    async fn try_multi_get_past_objects(
        &self,
        past_objects: Vec<SuiGetPastObjectRequest>,
        options: Option<SuiObjectDataOptions>,
    ) -> RpcResult<Vec<SuiPastObjectResponse>>;

    /// Return a checkpoint
    #[method(name = "getCheckpoint")]
    async fn get_checkpoint(&self, id: CheckpointId) -> RpcResult<Checkpoint>;

    /// Return paginated list of checkpoints
    #[method(name = "getCheckpoints")]
    async fn get_checkpoints(
        &self,
        cursor: Option<BigInt<u64>>,
        limit: Option<usize>,
        descending_order: bool,
    ) -> RpcResult<CheckpointPage>;

    /// Return transaction events.
    #[method(name = "getEvents")]
    async fn get_events(&self, transaction_digest: TransactionDigest) -> RpcResult<Vec<SuiEvent>>;

    /// Return the total number of transaction blocks known to the server.
    #[method(name = "getTotalTransactionBlocks")]
    async fn get_total_transaction_blocks(&self) -> RpcResult<BigInt<u64>>;

    /// Return the sequence number of the latest checkpoint that has been executed
    #[method(name = "getLatestCheckpointSequenceNumber")]
    async fn get_latest_checkpoint_sequence_number(&self) -> RpcResult<BigInt<u64>>;

    /// Return the protocol config table for the given version number.
    /// If the version number is not specified, If none is specified, the node uses the version of the latest epoch it has processed.
    #[method(name = "getProtocolConfig")]
    async fn get_protocol_config(
        &self,
        version: Option<BigInt<u64>>,
    ) -> RpcResult<ProtocolConfigResponse>;

    /// Return the first four bytes of the chain's genesis checkpoint digest.
    #[method(name = "getChainIdentifier")]
    async fn get_chain_identifier(&self) -> RpcResult<String>;
}
