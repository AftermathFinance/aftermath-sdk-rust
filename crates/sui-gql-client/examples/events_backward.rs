use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::queries::model::fragments::{EventEdge, EventFilter};
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://graphql.testnet.sui.io/graphql")]
    url: String,

    #[arg(long)]
    cursor: Option<String>,

    #[arg(
        long,
        default_value = "0xe76d8a37d4132278a7a752183e90e04890b9e7d0f6657eadb68821609a2a56a3::event::PriceFeedUpdateEvent"
    )]
    event_type: String,

    #[arg(long, default_value = "1")]
    page_size: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let Args {
        url,
        cursor,
        event_type,
        page_size,
    } = Args::parse();

    let client = ReqwestClient::new(reqwest::Client::default(), url);

    let filter = Some(EventFilter {
        sender: None,
        after_checkpoint: None,
        at_checkpoint: None,
        before_checkpoint: None,
        module: None,
        type_: Some(event_type),
    });

    let (events, _) = client.events_backward(filter, cursor, page_size).await?;

    for EventEdge { node, cursor } in events {
        println!("{node:#?}");
        println!("Cursor: {cursor}");
    }

    Ok(())
}
