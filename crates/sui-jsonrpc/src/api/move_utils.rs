// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use jsonrpsee::proc_macros::rpc;
use sui_sdk_types::Address;

use crate::msgs::{
    MoveFunctionArgType, SuiMoveNormalizedFunction, SuiMoveNormalizedModule,
    SuiMoveNormalizedStruct,
};

#[rpc(client, namespace = "sui")]
pub trait MoveUtils {
    /// Return the argument types of a Move function,
    /// based on normalized Type.
    #[method(name = "getMoveFunctionArgTypes")]
    async fn get_move_function_arg_types(
        &self,
        package: Address,
        module: String,
        function: String,
    ) -> RpcResult<Vec<MoveFunctionArgType>>;

    /// Return structured representations of all modules in the given package
    #[method(name = "getNormalizedMoveModulesByPackage")]
    async fn get_normalized_move_modules_by_package(
        &self,
        package: Address,
    ) -> RpcResult<BTreeMap<String, SuiMoveNormalizedModule>>;

    /// Return a structured representation of Move module
    #[method(name = "getNormalizedMoveModule")]
    async fn get_normalized_move_module(
        &self,
        package: Address,
        module_name: String,
    ) -> RpcResult<SuiMoveNormalizedModule>;

    /// Return a structured representation of Move struct
    #[method(name = "getNormalizedMoveStruct")]
    async fn get_normalized_move_struct(
        &self,
        package: Address,
        module_name: String,
        struct_name: String,
    ) -> RpcResult<SuiMoveNormalizedStruct>;

    /// Return a structured representation of Move function
    #[method(name = "getNormalizedMoveFunction")]
    async fn get_normalized_move_function(
        &self,
        package: Address,
        module_name: String,
        function_name: String,
    ) -> RpcResult<SuiMoveNormalizedFunction>;
}
