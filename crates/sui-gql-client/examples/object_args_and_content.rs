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
        .object_args_and_content(
            vec![
                "0x9b24a49b488960e80cfd951f7bebaddc0c7a7f9b223154b9ee625c0daac6dfa1".parse()?,
                "0x60d1a85f81172a7418206f4b16e1e07e40c91cf58783f63f18a25efc81442dcb".parse()?,
            ],
            true,
            None,
        )
        .await?;

    for (oarg, content) in result {
        println!("ObjectArg: {oarg:?}");
        println!("Content: {content:#?}");
    }
    Ok(())
}
