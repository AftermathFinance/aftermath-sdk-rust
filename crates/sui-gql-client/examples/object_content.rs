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
        .object_content(
            "0xe4a1c0bfc53a7c2941a433a9a681c942327278b402878e0c45280eecd098c3d1".parse()?,
            None,
        )
        .await?;
    println!("{result:#?}");
    Ok(())
}
