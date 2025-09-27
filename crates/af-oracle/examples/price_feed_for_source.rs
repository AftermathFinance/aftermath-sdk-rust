use af_oracle::graphql::GraphQlClientExt as _;
use af_sui_types::ObjectId;
use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,

    #[arg(long, default_value_t = af_sui_types::object_id(
        b"0x2e26816616244fe952ef924453d3468ed76addeaaf5873caf0970ba9b2b32722",
    ))]
    pfs: ObjectId,

    #[arg(long, default_value_t = af_sui_types::object_id(
        b"0x0280ab9931daa92ccbbd9798d271ea96a7c3551c77a5e0f04b1ba60b88822345",
    ))]
    source_wrapper_id: ObjectId,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let Args {
        rpc,
        pfs,
        source_wrapper_id,
    } = Args::parse();
    let client = ReqwestClient::new_default(rpc);

    let package = client.object_type(pfs).await?.address;
    if let Some(feed) = client
        .price_feed_for_source(package, pfs, source_wrapper_id)
        .await?
    {
        println!("Type: {:?}", feed.type_);
        println!("Value: {:?}", feed.value);
    }

    Ok(())
}
