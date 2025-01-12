use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::{EventEdge, EventFilter, GraphQlClientExt as _};
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    url: String,

    #[arg(long)]
    cursor: Option<String>,

    #[arg(
        long,
        default_value = "0xfd6f306bb2f8dce24dd3d4a9bdc51a46e7c932b15007d73ac0cfb38c15de0fea::events::AllocatedCollateral"
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
        transaction_digest: None,
        emitting_module: None,
        event_type: Some(event_type),
    });

    let (events, _) = client.events_backward(filter, cursor, page_size).await?;

    for EventEdge { node, cursor } in events {
        println!("{node:#?}");
        println!("Cursor: {cursor}");
    }

    Ok(())
}
