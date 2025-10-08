use color_eyre::Result;
use futures::TryStreamExt as _;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://graphql.testnet.sui.io/graphql";
const CH: &str = "0x20d1db9f07e7de98b290ea279fc21b927a37f7c084429a973773cd2187c3f914";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let client = ReqwestClient::new(
        reqwest::Client::default(),
        SUI_GRAPHQL_SERVER_URL.to_owned(),
    );

    tokio::pin!(
        let stream = client.object_dfs(CH.parse()?, None, None).await;
    );

    let mut count = 0;
    while let Some((name, value)) = stream.try_next().await? {
        count += 1;
        println!("Count: {count}");
        println!("Name: {name:?}, Value: {value:?}");
    }
    println!("Objects count: {count}");

    Ok(())
}
