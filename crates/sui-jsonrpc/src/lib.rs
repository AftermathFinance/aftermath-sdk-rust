// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! A fork of Mysten's `sui-json-rpc-api` and `sui-json-rpc-types` with minimal dependencies for client applications.

#[cfg(feature = "client-api")]
pub mod api;
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client-api")]
pub mod error;
pub mod msgs;
pub mod serde;

pub const RPC_QUERY_MAX_RESULT_LIMIT: &str = "RPC_QUERY_MAX_RESULT_LIMIT";
pub const DEFAULT_RPC_QUERY_MAX_RESULT_LIMIT: usize = 50;
pub const QUERY_MAX_RESULT_LIMIT_CHECKPOINTS: usize = 100;
pub const CLIENT_REQUEST_METHOD_HEADER: &str = "client-request-method";
pub const CLIENT_SDK_TYPE_HEADER: &str = "client-sdk-type";
/// The version number of the SDK itself. This can be different from the API version.
pub const CLIENT_SDK_VERSION_HEADER: &str = "client-sdk-version";
/// The RPC API version that the client is targeting. Different SDK versions may target the same
/// API version.
pub const CLIENT_TARGET_API_VERSION_HEADER: &str = "client-target-api-version";
pub const TRANSIENT_ERROR_CODE: i32 = -32050;
pub const TRANSACTION_EXECUTION_CLIENT_ERROR_CODE: i32 = -32002;
