use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://sui-testnet.mystenlabs.com/graphql";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let client = ReqwestClient::new(
        reqwest::Client::default(),
        SUI_GRAPHQL_SERVER_URL.to_owned(),
    );

    let curr_epoch_id = client.current_epoch_id().await?;

    for epoch_id in (curr_epoch_id.saturating_sub(10))..curr_epoch_id {
        let num = client.epoch_final_checkpoint_num(epoch_id).await?;
        println!("Epoch: {epoch_id}, Last checkpoint: {num}");
    }
    Ok(())
}
