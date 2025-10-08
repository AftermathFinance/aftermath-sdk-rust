use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://graphql.testnet.sui.io/graphql";
const COIN_TYPE: &str =
    "0xf79b0d6e1a7efcf7680749f63e2cc8887ce54fc5bcd986b0a557e358d524f22c::template::TEMPLATE";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let client = ReqwestClient::new(
        reqwest::Client::default(),
        SUI_GRAPHQL_SERVER_URL.to_owned(),
    );

    let result = client.coin_metadata(COIN_TYPE).await?;

    println!("{result:?}");
    Ok(())
}
