// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddr;

use af_sui_types::Address as SuiAddress;
use jsonrpsee::proc_macros::rpc;

use crate::msgs::{
    DevInspectArgs,
    DevInspectResults,
    DryRunTransactionBlockResponse,
    ExecuteTransactionRequestType,
    SuiTransactionBlockResponse,
    SuiTransactionBlockResponseOptions,
};
use crate::serde::BigInt;

#[rpc(client, namespace = "sui")]
pub trait WriteApi {
    /// Execute the transaction. See [`ExecuteTransactionRequestType`] for details on how it's
    /// handled by the RPC.
    ///
    /// `request_type` defaults to
    /// [`WaitForEffectsCert`](ExecuteTransactionRequestType::WaitForEffectsCert).
    #[method(name = "executeTransactionBlock")]
    async fn execute_transaction_block(
        &self,
        tx_bytes: String,
        signatures: Vec<String>,
        options: Option<SuiTransactionBlockResponseOptions>,
        request_type: Option<ExecuteTransactionRequestType>,
    ) -> RpcResult<SuiTransactionBlockResponse>;

    #[method(name = "monitoredExecuteTransactionBlock")]
    async fn monitored_execute_transaction_block(
        &self,
        tx_bytes: String,
        signatures: Vec<String>,
        options: Option<SuiTransactionBlockResponseOptions>,
        request_type: Option<ExecuteTransactionRequestType>,
        client_addr: Option<SocketAddr>,
    ) -> RpcResult<SuiTransactionBlockResponse>;

    /// Runs the transaction in dev-inspect mode. Which allows for nearly any
    /// transaction (or Move call) with any arguments. Detailed results are
    /// provided, including both the transaction effects and any return values.
    #[method(name = "devInspectTransactionBlock")]
    async fn dev_inspect_transaction_block(
        &self,
        sender_address: SuiAddress,
        tx_bytes: String,
        gas_price: Option<BigInt<u64>>,
        epoch: Option<BigInt<u64>>,
        additional_args: Option<DevInspectArgs>,
    ) -> RpcResult<DevInspectResults>;

    /// Return transaction execution effects including the gas cost summary,
    /// while the effects are not committed to the chain.
    #[method(name = "dryRunTransactionBlock")]
    async fn dry_run_transaction_block(
        &self,
        tx_bytes: String,
    ) -> RpcResult<DryRunTransactionBlockResponse>;
}
