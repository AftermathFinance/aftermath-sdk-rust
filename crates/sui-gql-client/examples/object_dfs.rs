use color_eyre::Result;
use futures::TryStreamExt as _;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://graphql.testnet.sui.io/graphql";
const CH: &str = "0x1d4fabd54c285e9019b57cfbbf2bf1c890155ae1348d3fbc10786ea71dac9913";

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
