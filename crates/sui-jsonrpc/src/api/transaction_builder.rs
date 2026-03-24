// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::too_many_arguments)]

use jsonrpsee::proc_macros::rpc;
use sui_sdk_types::Address;

use crate::msgs::{
    RPCTransactionRequestParams, SuiTransactionBlockBuilderMode, SuiTypeTag, TransactionBlockBytes,
};
use crate::serde::BigInt;

#[rpc(client, namespace = "unsafe")]
pub trait TransactionBuilder {
    /// Create an unsigned transaction to transfer an object from one address to another. The object's type
    /// must allow public transfers
    #[method(name = "transferObject")]
    async fn transfer_object(
        &self,
        signer: Address,
        object_id: Address,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
        recipient: Address,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Create an unsigned transaction to send SUI coin object to a Sui address. The SUI object is also used as the gas object.
    #[method(name = "transferSui")]
    async fn transfer_sui(
        &self,
        signer: Address,
        sui_object_id: Address,
        gas_budget: BigInt<u64>,
        recipient: Address,
        amount: Option<BigInt<u64>>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Send `Coin<T>` to a list of addresses, where `T` can be any coin type, following a list of amounts,
    /// The object specified in the `gas` field will be used to pay the gas fee for the transaction.
    /// The gas object can not appear in `input_coins`. If the gas object is not specified, the RPC server
    /// will auto-select one.
    #[method(name = "pay")]
    async fn pay(
        &self,
        signer: Address,
        input_coins: Vec<Address>,
        recipients: Vec<Address>,
        amounts: Vec<BigInt<u64>>,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Send SUI coins to a list of addresses, following a list of amounts.
    /// This is for SUI coin only and does not require a separate gas coin object.
    /// Specifically, what pay_sui does are:
    /// 1. debit each input_coin to create new coin following the order of
    /// amounts and assign it to the corresponding recipient.
    /// 2. accumulate all residual SUI from input coins left and deposit all SUI to the first
    /// input coin, then use the first input coin as the gas coin object.
    /// 3. the balance of the first input coin after tx is sum(input_coins) - sum(amounts) - actual_gas_cost
    /// 4. all other input coints other than the first one are deleted.
    #[method(name = "paySui")]
    async fn pay_sui(
        &self,
        signer: Address,
        input_coins: Vec<Address>,
        recipients: Vec<Address>,
        amounts: Vec<BigInt<u64>>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Send all SUI coins to one recipient.
    /// This is for SUI coin only and does not require a separate gas coin object.
    /// Specifically, what pay_all_sui does are:
    /// 1. accumulate all SUI from input coins and deposit all SUI to the first input coin
    /// 2. transfer the updated first coin to the recipient and also use this first coin as gas coin object.
    /// 3. the balance of the first input coin after tx is sum(input_coins) - actual_gas_cost.
    /// 4. all other input coins other than the first are deleted.
    #[method(name = "payAllSui")]
    async fn pay_all_sui(
        &self,
        signer: Address,
        input_coins: Vec<Address>,
        recipient: Address,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Create an unsigned transaction to execute a Move call on the network, by calling the specified function in the module of a given package.
    #[method(name = "moveCall")]
    async fn move_call(
        &self,
        signer: Address,
        package_object_id: Address,
        module: String,
        function: String,
        type_arguments: Vec<SuiTypeTag>,
        arguments: Vec<serde_json::Value>,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
        execution_mode: Option<SuiTransactionBlockBuilderMode>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Create an unsigned transaction to publish a Move package.
    #[method(name = "publish")]
    async fn publish(
        &self,
        sender: Address,
        compiled_modules: Vec<String>,
        dependencies: Vec<Address>,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Create an unsigned transaction to split a coin object into multiple coins.
    #[method(name = "splitCoin")]
    async fn split_coin(
        &self,
        signer: Address,
        coin_object_id: Address,
        split_amounts: Vec<BigInt<u64>>,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Create an unsigned transaction to split a coin object into multiple equal-size coins.
    #[method(name = "splitCoinEqual")]
    async fn split_coin_equal(
        &self,
        signer: Address,
        coin_object_id: Address,
        split_count: BigInt<u64>,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Create an unsigned transaction to merge multiple coins into one coin.
    #[method(name = "mergeCoins")]
    async fn merge_coin(
        &self,
        signer: Address,
        primary_coin: Address,
        coin_to_merge: Address,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Create an unsigned batched transaction.
    #[method(name = "batchTransaction")]
    async fn batch_transaction(
        &self,
        signer: Address,
        single_transaction_params: Vec<RPCTransactionRequestParams>,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
        txn_builder_mode: Option<SuiTransactionBlockBuilderMode>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Add stake to a validator's staking pool using multiple coins and amount.
    #[method(name = "requestAddStake")]
    async fn request_add_stake(
        &self,
        signer: Address,
        coins: Vec<Address>,
        amount: Option<BigInt<u64>>,
        validator: Address,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes>;

    /// Withdraw stake from a validator's staking pool.
    #[method(name = "requestWithdrawStake")]
    async fn request_withdraw_stake(
        &self,
        signer: Address,
        staked_sui: Address,
        gas: Option<Address>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes>;
}
