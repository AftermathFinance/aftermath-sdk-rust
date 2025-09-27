use std::time::Instant;

use af_iperps::graphql::GraphQlClientExt as _;
use af_sui_types::{ObjectId, hex_address_bytes};
use clap::Parser;
use color_eyre::Result;
use futures::TryStreamExt as _;
use indicatif::ProgressBar;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,

    #[arg(long, default_value_t = ObjectId::new(hex_address_bytes(
        b"0xa8d5e1bf15d606b589a8452e6864841bb7e5a56d031f77a387b2177dd32a1fcd",
    )))]
    map: ObjectId,

    /// Only the summary of query time and number of positions.
    #[arg(long, short)]
    summary: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc, map, summary } = Args::parse();
    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());

    tokio::pin!(
        let stream = client.map_orders(map, None);
    );

    let start = Instant::now();
    let spinner = spinner();
    let mut count = 0;
    while let Some((order_id, order)) = stream.try_next().await? {
        count += 1;
        if summary {
            spinner.tick();
        } else {
            println!("Order ID: {order_id}");
            println!("{order}");
        }
    }
    spinner.finish_using_style();
    println!("Elapsed: {:?}", Instant::now().duration_since(start));
    println!("Orders: {count}");
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
