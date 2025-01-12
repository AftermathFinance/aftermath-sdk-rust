use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://sui-testnet.mystenlabs.com/graphql";
const BTC_BIDS_MAP: &str = "0x5fb03a8666f451cef89758c9ad3247230436e46ee71544430124130558c91d5a";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let client = ReqwestClient::new(
        reqwest::Client::default(),
        SUI_GRAPHQL_SERVER_URL.to_owned(),
    );
    let mut dfs = vec![];
    let mut cursor = None;
    loop {
        let (page, next_cursor) = client
            .owner_df_contents(BTC_BIDS_MAP.parse()?, None, None, cursor)
            .await?;
        dfs.extend(page);
        if next_cursor.is_none() {
            break;
        }
        cursor = next_cursor;
    }
    for (name, value) in dfs {
        println!("{name}: {value}");
    }
    Ok(())
}
