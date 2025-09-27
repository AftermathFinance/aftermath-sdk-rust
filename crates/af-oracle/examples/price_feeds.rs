use std::time::Instant;

use af_oracle::graphql::GraphQlClientExt as _;
use af_sui_types::ObjectId;
use clap::Parser;
use color_eyre::Result;
use futures::TryStreamExt as _;
use indicatif::ProgressBar;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,

    #[arg(long, default_value_t = af_sui_types::object_id(
        b"0x2e26816616244fe952ef924453d3468ed76addeaaf5873caf0970ba9b2b32722",
    ))]
    pfs: ObjectId,

    /// Only the summary of query time and number of price feeds.
    #[arg(long, short)]
    summary: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc, pfs, summary } = Args::parse();
    let client = ReqwestClient::new_default(rpc);

    tokio::pin!(
        let stream = client.price_feeds(pfs, None);
    );

    let start = Instant::now();
    let spinner = spinner();
    let mut count = 0;
    while let Some(price_feed) = stream.try_next().await? {
        count += 1;
        if !summary {
            println!("Source wrapper id {}", price_feed.0);
            println!("{}", price_feed.1.value);
        } else {
            spinner.tick();
        }
    }
    spinner.finish_using_style();
    println!("Elapsed {:?}", Instant::now().duration_since(start));
    println!("Price Feeds: {count}");
    Ok(())
}

// https://github.com/console-rs/indicatif/blob/main/examples/long-spinner.rs
fn spinner() -> ProgressBar {
    use indicatif::{ProgressFinish, ProgressStyle};
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .expect("init spinner")
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    pb.set_message("Querying...");
    pb.with_finish(ProgressFinish::Abandon)
}
