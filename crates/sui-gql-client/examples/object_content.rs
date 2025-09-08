use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://sui-testnet.mystenlabs.com/graphql";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let client = ReqwestClient::new(
        reqwest::Client::default(),
        SUI_GRAPHQL_SERVER_URL.to_owned(),
    );
    let result = client
        .full_object(
            "0xa8f4b35457e5ecfa77d448858ad9ae9330541785046726473e84da69f9557339".parse()?,
            None,
        )
        .await?;
    println!("{result:#?}");
    Ok(())
}
