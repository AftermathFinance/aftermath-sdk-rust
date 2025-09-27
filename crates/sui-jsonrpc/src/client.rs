// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;

use af_sui_types::{
    Address as SuiAddress,
    GasCostSummary,
    GasData,
    Object,
    ObjectArg,
    ObjectId,
    ObjectRef,
    TransactionData,
    TransactionDataV1,
    TransactionExpiration,
    TransactionKind,
    UserSignature,
    encode_base64_default,
};
use futures_core::Stream;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::{PingConfig, WsClient, WsClientBuilder};
use jsonrpsee_http_client::{HeaderMap, HeaderValue, HttpClient, HttpClientBuilder};
use serde_json::Value;

use super::{CLIENT_SDK_TYPE_HEADER, CLIENT_SDK_VERSION_HEADER, CLIENT_TARGET_API_VERSION_HEADER};
use crate::api::{CoinReadApiClient, ReadApiClient as _, WriteApiClient as _};
use crate::error::JsonRpcClientError;
use crate::msgs::{
    Coin,
    DryRunTransactionBlockResponse,
    SuiExecutionStatus,
    SuiObjectData,
    SuiObjectDataError,
    SuiObjectDataOptions,
    SuiObjectResponse,
    SuiObjectResponseError,
    SuiObjectResponseQuery,
    SuiTransactionBlockEffectsAPI as _,
    SuiTransactionBlockResponse,
    SuiTransactionBlockResponseOptions,
};

/// Maximum possible budget.
pub const MAX_GAS_BUDGET: u64 = 50000000000;
/// Maximum number of objects that can be fetched via
/// [multi_get_objects][crate::api::ReadApiClient::multi_get_objects].
pub const MULTI_GET_OBJECT_MAX_SIZE: usize = 50;
pub const SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub const SUI_LOCAL_NETWORK_URL: &str = "http://127.0.0.1:9000";
pub const SUI_LOCAL_NETWORK_WS: &str = "ws://127.0.0.1:9000";
pub const SUI_LOCAL_NETWORK_GAS_URL: &str = "http://127.0.0.1:5003/gas";
pub const SUI_DEVNET_URL: &str = "https://fullnode.devnet.sui.io:443";
pub const SUI_TESTNET_URL: &str = "https://fullnode.testnet.sui.io:443";

pub type SuiClientResult<T = ()> = Result<T, SuiClientError>;
// Placeholder for errors that can't be added to `SuiClientError` yet since that would be a
// breaking change.
type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(thiserror::Error, Debug)]
pub enum SuiClientError {
    #[error("jsonrpsee client error: {0}")]
    JsonRpcClient(#[from] JsonRpcClientError),
    #[error("Data error: {0}")]
    DataError(String),
    #[error(
        "Client/Server api version mismatch, client api version : {client_version}, server api version : {server_version}"
    )]
    ServerVersionMismatch {
        client_version: String,
        server_version: String,
    },
    #[error(
        "Insufficient funds for address [{address}]; found balance {found}, requested: {requested}"
    )]
    InsufficientFunds {
        address: SuiAddress,
        found: u64,
        requested: u64,
    },
    #[error("In object response: {0}")]
    SuiObjectResponse(#[from] SuiObjectResponseError),
    #[error("In object data: {0}")]
    SuiObjectData(#[from] SuiObjectDataError),
}

/// A Sui client builder for connecting to the Sui network
///
/// By default the `maximum concurrent requests` is set to 256 and
/// the `request timeout` is set to 60 seconds. These can be adjusted using the
/// `max_concurrent_requests` function, and the `request_timeout` function.
/// If you use the WebSocket, consider setting the `ws_ping_interval` field to a
/// value of your choice to prevent the inactive WS subscription being
/// disconnected due to proxy timeout.
///
/// # Examples
///
/// ```rust,no_run
/// use sui_jsonrpc::client::SuiClientBuilder;
/// #[tokio::main]
/// async fn main() -> Result<(), color_eyre::eyre::Error> {
///     let sui = SuiClientBuilder::default()
///         .build("http://127.0.0.1:9000")
///         .await?;
///
///     println!("Sui local network version: {:?}", sui.api_version());
///     Ok(())
/// }
/// ```
pub struct SuiClientBuilder {
    request_timeout: Duration,
    ws_url: Option<String>,
    ws_ping_interval: Option<Duration>,
    basic_auth: Option<(String, String)>,
}

impl Default for SuiClientBuilder {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(60),
            ws_url: None,
            ws_ping_interval: None,
            basic_auth: None,
        }
    }
}

impl SuiClientBuilder {
    /// Set the request timeout to the specified duration
    pub const fn request_timeout(mut self, request_timeout: Duration) -> Self {
        self.request_timeout = request_timeout;
        self
    }

    /// Set the WebSocket URL for the Sui network
    #[deprecated = "\
        JSON-RPC subscriptions have been deprecated since at least mainnet-v1.28.3. \
        See <https://github.com/MystenLabs/sui/releases/tag/mainnet-v1.28.3>\
    "]
    pub fn ws_url(mut self, url: impl AsRef<str>) -> Self {
        self.ws_url = Some(url.as_ref().to_string());
        self
    }

    /// Set the WebSocket ping interval
    #[deprecated = "\
        JSON-RPC subscriptions have been deprecated since at least mainnet-v1.28.3. \
        See <https://github.com/MystenLabs/sui/releases/tag/mainnet-v1.28.3>\
    "]
    pub const fn ws_ping_interval(mut self, duration: Duration) -> Self {
        self.ws_ping_interval = Some(duration);
        self
    }

    /// Set the basic auth credentials for the HTTP client
    pub fn basic_auth(mut self, username: impl AsRef<str>, password: impl AsRef<str>) -> Self {
        self.basic_auth = Some((username.as_ref().to_string(), password.as_ref().to_string()));
        self
    }

    /// Returns a [SuiClient] object that is ready to interact with the local
    /// development network (by default it expects the Sui network to be
    /// up and running at `127.0.0.1:9000`).
    pub async fn build_localnet(self) -> SuiClientResult<SuiClient> {
        self.build(SUI_LOCAL_NETWORK_URL).await
    }

    /// Returns a [SuiClient] object that is ready to interact with the Sui devnet.
    pub async fn build_devnet(self) -> SuiClientResult<SuiClient> {
        self.build(SUI_DEVNET_URL).await
    }

    /// Returns a [SuiClient] object that is ready to interact with the Sui testnet.
    pub async fn build_testnet(self) -> SuiClientResult<SuiClient> {
        self.build(SUI_TESTNET_URL).await
    }

    /// Returns a [SuiClient] object connected to the Sui network running at the URI provided.
    #[allow(clippy::future_not_send)]
    pub async fn build(self, http: impl AsRef<str>) -> SuiClientResult<SuiClient> {
        let client_version = env!("CARGO_PKG_VERSION");
        let mut headers = HeaderMap::new();
        headers.insert(
            CLIENT_TARGET_API_VERSION_HEADER,
            // in rust, the client version is the same as the target api version
            HeaderValue::from_static(client_version),
        );
        headers.insert(
            CLIENT_SDK_VERSION_HEADER,
            HeaderValue::from_static(client_version),
        );
        headers.insert(CLIENT_SDK_TYPE_HEADER, HeaderValue::from_static("rust"));

        if let Some((username, password)) = self.basic_auth {
            let auth = encode_base64_default(format!("{}:{}", username, password));
            headers.insert(
                http::header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {}", auth))
                    .expect("Failed creating HeaderValue for basic auth"),
            );
        }

        let ws = if let Some(url) = self.ws_url {
            let mut builder = WsClientBuilder::default()
                .max_request_size(2 << 30)
                .set_headers(headers.clone())
                .request_timeout(self.request_timeout);

            if let Some(duration) = self.ws_ping_interval {
                builder = builder.enable_ws_ping(PingConfig::default().ping_interval(duration))
            }

            Some(builder.build(url).await?)
        } else {
            None
        };

        let http = HttpClientBuilder::default()
            .max_request_size(2 << 30)
            .set_headers(headers.clone())
            .request_timeout(self.request_timeout)
            .build(http)?;

        let info = Self::get_server_info(&http, &ws).await?;

        Ok(SuiClient {
            http: Arc::new(http),
            ws: Arc::new(ws),
            info: Arc::new(info),
        })
    }

    /// Return the server information as a `ServerInfo` structure.
    ///
    /// Fails with an error if it cannot call the RPC discover.
    async fn get_server_info(
        http: &HttpClient,
        ws: &Option<WsClient>,
    ) -> Result<ServerInfo, SuiClientError> {
        let rpc_spec: Value = http.request("rpc.discover", rpc_params![]).await?;
        let version = rpc_spec
            .pointer("/info/version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SuiClientError::DataError(
                    "Fail parsing server version from rpc.discover endpoint.".into(),
                )
            })?;
        let rpc_methods = Self::parse_methods(&rpc_spec)?;

        let subscriptions = if let Some(ws) = ws {
            let rpc_spec: Value = ws.request("rpc.discover", rpc_params![]).await?;
            Self::parse_methods(&rpc_spec)?
        } else {
            Vec::new()
        };
        Ok(ServerInfo {
            rpc_methods,
            subscriptions,
            version: version.to_string(),
        })
    }

    fn parse_methods(server_spec: &Value) -> Result<Vec<String>, SuiClientError> {
        let methods = server_spec
            .pointer("/methods")
            .and_then(|methods| methods.as_array())
            .ok_or_else(|| {
                SuiClientError::DataError(
                    "Fail parsing server information from rpc.discover endpoint.".into(),
                )
            })?;

        Ok(methods
            .iter()
            .flat_map(|method| method["name"].as_str())
            .map(|s| s.into())
            .collect())
    }
}

/// SuiClient is the basic type that provides all the necessary abstractions for interacting with the Sui network.
///
/// # Usage
///
/// Use [SuiClientBuilder] to build a [SuiClient].
#[derive(Clone)]
pub struct SuiClient {
    http: Arc<HttpClient>,
    ws: Arc<Option<WsClient>>,
    info: Arc<ServerInfo>,
}

impl Debug for SuiClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RPC client. Http: {:?}, Websocket: {:?}",
            self.http, self.ws
        )
    }
}

/// ServerInfo contains all the useful information regarding the API version, the available RPC calls, and subscriptions.
struct ServerInfo {
    rpc_methods: Vec<String>,
    subscriptions: Vec<String>,
    version: String,
}

impl SuiClient {
    pub fn builder() -> SuiClientBuilder {
        Default::default()
    }

    /// Returns a list of RPC methods supported by the node the client is connected to.
    #[rustversion::attr(
        stable,
        expect(
            clippy::missing_const_for_fn,
            reason = "Not changing the public API right now"
        )
    )]
    pub fn available_rpc_methods(&self) -> &Vec<String> {
        &self.info.rpc_methods
    }

    /// Returns a list of streaming/subscription APIs supported by the node the client is connected to.
    #[rustversion::attr(
        stable,
        expect(
            clippy::missing_const_for_fn,
            reason = "Not changing the public API right now"
        )
    )]
    pub fn available_subscriptions(&self) -> &Vec<String> {
        &self.info.subscriptions
    }

    /// Returns the API version information as a string.
    ///
    /// The format of this string is `<major>.<minor>.<patch>`, e.g., `1.6.0`,
    /// and it is retrieved from the OpenRPC specification via the discover service method.
    #[rustversion::attr(
        stable,
        expect(
            clippy::missing_const_for_fn,
            reason = "Not changing the public API right now"
        )
    )]
    pub fn api_version(&self) -> &str {
        &self.info.version
    }

    /// Verifies if the API version matches the server version and returns an error if they do not match.
    pub fn check_api_version(&self) -> SuiClientResult<()> {
        let server_version = self.api_version();
        let client_version = env!("CARGO_PKG_VERSION");
        if server_version != client_version {
            return Err(SuiClientError::ServerVersionMismatch {
                client_version: client_version.to_string(),
                server_version: server_version.to_string(),
            });
        };
        Ok(())
    }

    /// Returns a reference to the underlying http client.
    #[rustversion::attr(
        stable,
        expect(
            clippy::missing_const_for_fn,
            reason = "Not changing the public API right now"
        )
    )]
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Returns a reference to the underlying WebSocket client, if any.
    #[rustversion::attr(
        stable,
        expect(
            clippy::missing_const_for_fn,
            reason = "Not changing the public API right now"
        )
    )]
    pub fn ws(&self) -> Option<&WsClient> {
        (*self.ws).as_ref()
    }

    pub async fn get_shared_oarg(&self, id: ObjectId, mutable: bool) -> SuiClientResult<ObjectArg> {
        let data = self
            .http()
            .get_object(id, Some(SuiObjectDataOptions::new().with_owner()))
            .await?
            .into_object()?;
        Ok(data.shared_object_arg(mutable)?)
    }

    pub async fn get_imm_or_owned_oarg(&self, id: ObjectId) -> SuiClientResult<ObjectArg> {
        let data = self
            .http()
            .get_object(id, Some(SuiObjectDataOptions::new().with_owner()))
            .await?
            .into_object()?;
        Ok(data.imm_or_owned_object_arg()?)
    }

    /// Query the PTB args for several objects.
    ///
    /// Mutability (for shared objects) defaults to `false`.
    ///
    /// This has **no** consistency guarantees, i.e., object versions may come from different
    /// points in the chain's history (i.e., from different checkpoints).
    pub async fn object_args<Iter>(
        &self,
        ids: Iter,
    ) -> Result<impl Iterator<Item = Result<ObjectArg, BoxError>> + use<Iter>, BoxError>
    where
        Iter: IntoIterator<Item = ObjectId> + Send,
        Iter::IntoIter: Send,
    {
        let options = SuiObjectDataOptions::new().with_owner();
        Ok(self
            .multi_get_objects(ids, options)
            .await?
            .into_iter()
            .map(|r| Ok(r.into_object()?.object_arg(false)?)))
    }

    /// Query the full object contents as a standard Sui type.
    pub async fn full_object(&self, id: ObjectId) -> Result<Object, BoxError> {
        let options = SuiObjectDataOptions {
            show_bcs: true,
            show_owner: true,
            show_storage_rebate: true,
            show_previous_transaction: true,
            ..Default::default()
        };
        Ok(self
            .http()
            .get_object(id, Some(options))
            .await?
            .into_object()?
            .into_full_object()?)
    }

    /// Query the full contents for several objects.
    ///
    /// This has **no** consistency guarantees, i.e., object versions may come from different
    /// points in the chain's history (i.e., from different checkpoints).
    pub async fn full_objects<Iter>(
        &self,
        ids: Iter,
    ) -> Result<impl Iterator<Item = Result<Object, BoxError>>, BoxError>
    where
        Iter: IntoIterator<Item = ObjectId> + Send,
        Iter::IntoIter: Send,
    {
        let options = SuiObjectDataOptions {
            show_bcs: true,
            show_owner: true,
            show_storage_rebate: true,
            show_previous_transaction: true,
            ..Default::default()
        };
        Ok(self
            .multi_get_objects(ids, options)
            .await?
            .into_iter()
            .map(|r| Ok(r.into_object()?.into_full_object()?)))
    }

    /// Return the object data for a list of objects.
    ///
    /// This method works for any number of object ids.
    pub async fn multi_get_objects<I>(
        &self,
        object_ids: I,
        options: SuiObjectDataOptions,
    ) -> SuiClientResult<Vec<SuiObjectResponse>>
    where
        I: IntoIterator<Item = ObjectId> + Send,
        I::IntoIter: Send,
    {
        let mut result = Vec::new();
        for chunk in iter_chunks(object_ids, MULTI_GET_OBJECT_MAX_SIZE) {
            if chunk.len() == 1 {
                let elem = self
                    .http()
                    .get_object(chunk[0], Some(options.clone()))
                    .await?;
                result.push(elem);
            } else {
                let it = self
                    .http()
                    .multi_get_objects(chunk, Some(options.clone()))
                    .await?;
                result.extend(it);
            }
        }
        Ok(result)
    }

    /// Helper to execute a transaction using standard Sui types.
    ///
    /// See [`JsonRpcClientErrorExt::as_error_object`] and [`ErrorObjectExt`] for how to inspect
    /// the error in case a transaction failed before being included in a checkpoint.
    ///
    /// [`JsonRpcClientErrorExt::as_error_object`]: crate::error::JsonRpcClientErrorExt::as_error_object
    /// [`ErrorObjectExt`]: crate::error::ErrorObjectExt
    pub async fn submit_transaction(
        &self,
        tx_data: &TransactionData,
        signatures: &[UserSignature],
        options: Option<SuiTransactionBlockResponseOptions>,
    ) -> Result<SuiTransactionBlockResponse, JsonRpcClientError> {
        self.http()
            .execute_transaction_block(
                tx_data.encode_base64(),
                signatures.iter().map(UserSignature::to_base64).collect(),
                options,
                None,
            )
            .await
    }

    /// Helper to dry run a transaction kind for its effects.
    ///
    /// This sets the gas budget to the maximum possible. If you want to test different values,
    /// manually perform the dry-run using the inner [`Self::http`] client.
    pub async fn dry_run_transaction(
        &self,
        tx_kind: &TransactionKind,
        sender: SuiAddress,
        gas_price: u64,
    ) -> Result<DryRunTransactionBlockResponse, JsonRpcClientError> {
        let sentinel = TransactionData::V1(TransactionDataV1 {
            kind: tx_kind.clone(),
            sender,
            gas_data: GasData {
                payment: vec![],
                owner: sender,
                price: gas_price,
                budget: MAX_GAS_BUDGET,
            },
            expiration: TransactionExpiration::None,
        });
        self.http()
            .dry_run_transaction_block(sentinel.encode_base64())
            .await
    }

    /// Estimate a budget for the transaction by dry-running it.
    ///
    /// Uses default [`GasBudgetOptions`] to compute the cost estimate.
    pub async fn gas_budget(
        &self,
        tx_kind: &TransactionKind,
        sender: SuiAddress,
        price: u64,
    ) -> Result<u64, DryRunError> {
        let options = GasBudgetOptions::new(price);
        self.gas_budget_with_options(tx_kind, sender, options).await
    }

    /// Estimate a budget for the transaction by dry-running it.
    pub async fn gas_budget_with_options(
        &self,
        tx_kind: &TransactionKind,
        sender: SuiAddress,
        options: GasBudgetOptions,
    ) -> Result<u64, DryRunError> {
        let sentinel = TransactionData::V1(TransactionDataV1 {
            kind: tx_kind.clone(),
            sender,
            gas_data: GasData {
                payment: vec![],
                owner: sender,
                price: options.price,
                budget: options.dry_run_budget,
            },
            expiration: TransactionExpiration::None,
        });
        let response = self
            .http()
            .dry_run_transaction_block(encode_base64_default(
                bcs::to_bytes(&sentinel).expect("TransactionData serialization shouldn't fail"),
            ))
            .await?;
        if let SuiExecutionStatus::Failure { error } = response.effects.status() {
            return Err(DryRunError::Execution(error.clone(), response));
        }

        let budget = {
            let safe_overhead = options.safe_overhead_multiplier * options.price;
            estimate_gas_budget_from_gas_cost(response.effects.gas_cost_summary(), safe_overhead)
        };
        Ok(budget)
    }

    /// Build the gas data for a transaction by querying the node for gas objects.
    pub async fn get_gas_data(
        &self,
        tx_kind: &TransactionKind,
        sponsor: SuiAddress,
        budget: u64,
        price: u64,
    ) -> Result<GasData, GetGasDataError> {
        let exclude = if let TransactionKind::ProgrammableTransaction(ptb) = tx_kind {
            use sui_sdk_types::Input::*;

            ptb.inputs
                .iter()
                .filter_map(|i| match i {
                    Pure { .. } => None,
                    Shared { object_id, .. } => Some(*object_id),
                    ImmutableOrOwned(oref) | Receiving(oref) => Some(*oref.object_id()),
                })
                .collect()
        } else {
            vec![]
        };

        if budget < price {
            return Err(GetGasDataError::BudgetTooSmall { budget, price });
        }

        let payment = self
            .get_gas_payment(sponsor, budget, &exclude)
            .await
            .map_err(GetGasDataError::from_not_enough_gas)?;

        Ok(GasData {
            payment,
            owner: sponsor,
            price,
            budget,
        })
    }

    /// Query the node for gas objects to fulfill a certain budget.
    ///
    /// `exclude`s certain object ids from being part of the returned objects.
    pub async fn get_gas_payment(
        &self,
        sponsor: SuiAddress,
        budget: u64,
        exclude: &[ObjectId],
    ) -> Result<Vec<ObjectRef>, NotEnoughGasError> {
        Ok(self
            .coins_for_amount(sponsor, Some("0x2::sui::SUI".to_owned()), budget, exclude)
            .await
            .map_err(|inner| NotEnoughGasError {
                sponsor,
                budget,
                inner,
            })?
            .into_iter()
            .map(|c| c.object_ref())
            .collect())
    }

    #[deprecated(since = "0.14.5", note = "use SuiClient::coins_for_amount")]
    pub async fn select_coins(
        &self,
        address: SuiAddress,
        coin_type: Option<String>,
        amount: u64,
        exclude: Vec<ObjectId>,
    ) -> SuiClientResult<Vec<Coin>> {
        self.coins_for_amount(address, coin_type, amount, &exclude)
            .await
    }

    /// Return a list of coins for the given address, or an error upon failure.
    ///
    /// Note that the function selects coins to meet or exceed the requested `amount`.
    /// If that it is not possible, it will fail with an insufficient fund error.
    ///
    /// The coins can be filtered by `coin_type` (e.g., 0x168da5bf1f48dafc111b0a488fa454aca95e0b5e::usdc::USDC)
    /// or use `None` to use the default `Coin<SUI>`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sui_jsonrpc::client::SuiClientBuilder;
    /// use af_sui_types::Address as SuiAddress;
    ///
    /// #[tokio::main]
    /// async fn main() -> color_eyre::Result<()> {
    ///     let sui = SuiClientBuilder::default().build_localnet().await?;
    ///     let address = "0x0000....0000".parse()?;
    ///     let coins = sui
    ///         .select_coins(address, None, 5, vec![])
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn coins_for_amount(
        &self,
        address: SuiAddress,
        coin_type: Option<String>,
        amount: u64,
        exclude: &[ObjectId],
    ) -> SuiClientResult<Vec<Coin>> {
        use futures_util::{TryStreamExt as _, future};
        let mut coins = vec![];
        let mut total = 0;
        let mut stream = std::pin::pin!(
            self.coins_for_address(address, coin_type, None)
                .try_filter(|c| future::ready(!exclude.contains(&c.coin_object_id)))
        );

        while let Some(coin) = stream.try_next().await? {
            total += coin.balance;
            coins.push(coin);
            if total >= amount {
                return Ok(coins);
            }
        }

        Err(SuiClientError::InsufficientFunds {
            address,
            found: total,
            requested: amount,
        })
    }

    /// Return a stream of coins for the given address, or an error upon failure.
    ///
    /// This simply wraps a paginated query. Use `page_size` to control the inner query's page
    /// size.
    ///
    /// The coins can be filtered by `coin_type` (e.g., 0x168da5bf1f48dafc111b0a488fa454aca95e0b5e::usdc::USDC)
    /// or use `None` to use the default `Coin<SUI>`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sui_jsonrpc::client::SuiClientBuilder;
    /// use af_sui_types::Address as SuiAddress;
    /// use futures::TryStreamExt as _;
    ///
    /// #[tokio::main]
    /// async fn main() -> color_eyre::Result<()> {
    ///     let sui = SuiClientBuilder::default().build_localnet().await?;
    ///     let address = "0x0000....0000".parse()?;
    ///     let mut coins = std::pin::pin!(sui.coins_for_address(address, None, Some(5)));
    ///
    ///     while let Some(coin) = coins.try_next().await? {
    ///         println!("{coin:?}");
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn coins_for_address(
        &self,
        address: SuiAddress,
        coin_type: Option<String>,
        page_size: Option<u32>,
    ) -> impl Stream<Item = SuiClientResult<Coin>> + Send + '_ {
        async_stream::try_stream! {
            let mut has_next_page = true;
            let mut cursor = None;

            while has_next_page {
                let page = self
                    .http()
                    .get_coins(address, coin_type.clone(), cursor, page_size.map(|u| u as usize))
                    .await?;

                for coin in page.data
                {
                    yield coin;
                }

                has_next_page = page.has_next_page;
                cursor = page.next_cursor;
            }
        }
    }

    /// Return a stream of objects owned by the given address.
    ///
    /// This simply wraps a paginated query. Use `page_size` to control the inner query's page
    /// size.
    pub fn owned_objects(
        &self,
        owner: SuiAddress,
        query: Option<SuiObjectResponseQuery>,
        page_size: Option<u32>,
    ) -> impl Stream<Item = SuiClientResult<SuiObjectData>> + Send + '_ {
        use crate::api::IndexerApiClient as _;
        async_stream::try_stream! {
            let mut has_next_page = true;
            let mut cursor = None;

            while has_next_page {
                let page = self
                    .http()
                    .get_owned_objects(owner, query.clone(), cursor, page_size.map(|u| u as usize)).await?;

                for data in page.data {
                    yield data.into_object()?;
                }
                has_next_page = page.has_next_page;
                cursor = page.next_cursor;
            }
        }
    }

    /// Get the latest object reference for an ID from the node.
    pub async fn latest_object_ref(&self, object_id: ObjectId) -> SuiClientResult<ObjectRef> {
        Ok(self
            .http()
            .get_object(object_id, Some(SuiObjectDataOptions::default()))
            .await?
            .into_object()?
            .object_ref())
    }
}

/// Parameters for computing the gas budget for a transaction using a dry-run.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct GasBudgetOptions {
    /// The gas price. Must be set via [`Self::new`].
    pub price: u64,

    /// The budget for the dry-run.
    pub dry_run_budget: u64,

    /// Multiplier on the gas price. The result is a balance to add to both the computation and net
    /// gas costs to account for possible fluctuations when the transaction is actually submitted.
    pub safe_overhead_multiplier: u64,
}

impl GasBudgetOptions {
    #[expect(
        clippy::missing_const_for_fn,
        reason = "We might evolve the defaults to use non-const expressions"
    )]
    pub fn new(price: u64) -> Self {
        Self {
            price,
            dry_run_budget: MAX_GAS_BUDGET,
            safe_overhead_multiplier: GAS_SAFE_OVERHEAD_MULTIPLIER,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "Boxing now would break backwards compatibility"
)]
pub enum DryRunError {
    #[error("Error in dry run: {0}")]
    Execution(String, DryRunTransactionBlockResponse),
    #[error("In JSON-RPC client: {0}")]
    Client(#[from] JsonRpcClientError),
}

#[derive(thiserror::Error, Debug)]
pub enum GetGasDataError {
    #[error("In JSON-RPC client: {0}")]
    Client(#[from] JsonRpcClientError),
    #[error(
        "Gas budget {budget} is less than the gas price {price}. \
           The gas budget must be at least the gas price of {price}."
    )]
    BudgetTooSmall { budget: u64, price: u64 },
    #[error(
        "Cannot find gas coins for address [{sponsor}] \
        with amount sufficient for the required gas amount [{budget}]. \
        Caused by {inner}"
    )]
    NotEnoughGas {
        sponsor: SuiAddress,
        budget: u64,
        inner: SuiClientError,
    },
}

impl GetGasDataError {
    fn from_not_enough_gas(e: NotEnoughGasError) -> Self {
        let NotEnoughGasError {
            sponsor,
            budget,
            inner,
        } = e;
        Self::NotEnoughGas {
            sponsor,
            budget,
            inner,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error(
    "Cannot find gas coins for address [{sponsor}] \
        with amount sufficient for the required gas amount [{budget}]. \
        Caused by {inner}"
)]
pub struct NotEnoughGasError {
    sponsor: SuiAddress,
    budget: u64,
    inner: SuiClientError,
}

/// Multiplier on the gas price for computing gas budgets from dry-runs.
///
/// Same value as used in the Sui CLI.
const GAS_SAFE_OVERHEAD_MULTIPLIER: u64 = 1000;

/// Use primarily on the gas cost of dry-runs.
///
/// Same as used in the Sui CLI.
///
/// # Arguments
/// - `gas_cost_summary`: gas cost breakdown
/// - `safe_overhead`: balance to add to both the computation and net gas costs to account for
///   possible fluctuations when the transaction is actually submitted.
fn estimate_gas_budget_from_gas_cost(gas_cost_summary: &GasCostSummary, safe_overhead: u64) -> u64 {
    let computation_cost_with_overhead = gas_cost_summary.computation_cost + safe_overhead;

    let gas_usage_with_overhead = gas_cost_summary.net_gas_usage() + safe_overhead as i64;
    computation_cost_with_overhead.max(gas_usage_with_overhead.max(0) as u64)
}

fn iter_chunks<I>(iter: I, chunk_size: usize) -> impl Iterator<Item = Vec<I::Item>> + Send
where
    I: IntoIterator,
    I::IntoIter: Send,
{
    let mut iter = iter.into_iter();
    std::iter::from_fn(move || {
        let elem = iter.next()?;
        let mut v = Vec::with_capacity(chunk_size);
        v.push(elem);
        v.extend(iter.by_ref().take(chunk_size - 1));
        Some(v)
    })
}
