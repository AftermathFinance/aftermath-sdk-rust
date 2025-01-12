use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use clap::Parser;
use color_eyre::Result;
use cynic::Operation;
use serde::Serialize;
use serde_json::Value as Json;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;
use sui_gql_client::RawClient;

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Cli { rpc } = Cli::parse();

    let inner = ReqwestClient::new_default(rpc);
    let client = CachedClient {
        inner: &inner,
        cache: Default::default(),
    };

    let ckpt_num = client.latest_checkpoint().await?;
    println!("Checkpoint: {ckpt_num}");
    tokio::time::sleep(Duration::from_secs(2)).await;
    let ckpt_num = client.latest_checkpoint().await?;
    println!("Checkpoint: {ckpt_num}");
    Ok(())
}

type Map = HashMap<(String, Json), Json>;

type Cache = Mutex<Map>;

pub struct CachedClient<'a, T> {
    inner: &'a T,
    cache: Cache,
}

#[derive(thiserror::Error, Debug)]
pub enum Error<I> {
    #[error(transparent)]
    Inner(I),
    #[error("Serializing variables: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Poisoned mutex")]
    Mutex(String),
}

impl<T> RawClient for CachedClient<'_, T>
where
    T: RawClient + Sync,
    T::Error: std::error::Error,
{
    type Error = Error<T::Error>;

    async fn run_graphql_raw<Query, Vars>(
        &self,
        operation: Operation<Query, Vars>,
    ) -> Result<Json, Self::Error>
    where
        Vars: Serialize + Send,
    {
        let key = (
            operation.query.clone(),
            serde_json::to_value(&operation.variables)?,
        );
        let cached = self
            .cache
            .lock()
            .map_err(|e| Error::Mutex(e.to_string()))?
            .get(&key)
            .cloned();
        if let Some(result) = cached {
            Ok(result)
        } else {
            let result = self
                .inner
                .run_graphql_raw(operation)
                .await
                .map_err(Error::Inner)?;
            self.cache
                .lock()
                .map_err(|e| Error::Mutex(e.to_string()))?
                .insert(key, result.clone());
            Ok(result)
        }
    }
}
