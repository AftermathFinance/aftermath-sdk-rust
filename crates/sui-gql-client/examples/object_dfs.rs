use color_eyre::Result;
use futures::TryStreamExt as _;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://graphql.testnet.sui.io/graphql";
const CH: &str = "0xf6f30ee0450f6e3e628b68ac473699f26da5063f74be1868155a8a83b8b45060";

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
        println!("Name: {name:?}, Value: {value:?}");
    }
    println!("Objects count: {count}");

    Ok(())
}
