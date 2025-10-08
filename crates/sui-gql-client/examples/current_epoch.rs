use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://graphql.testnet.sui.io/graphql";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let client = ReqwestClient::new(
        reqwest::Client::default(),
        SUI_GRAPHQL_SERVER_URL.to_owned(),
    );

    let (epoch_id, rgp) = client.current_epoch().await?;
    println!("Epoch id: {epoch_id}, RGP: {rgp}");

    Ok(())
}
