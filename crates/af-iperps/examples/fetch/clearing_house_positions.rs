//! Since this query is a [`Stream`], positions will be printed as they arrive.
//!
//! This is quickly fill your terminal screen.
//!
//! [`Stream`]: futures::Stream

use std::time::Instant;

use af_iperps::graphql::GraphQlClientExt as _;
use af_sui_types::Address;
use clap::Parser;
use color_eyre::Result;
use futures::TryStreamExt as _;
use indicatif::ProgressBar;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://graphql.testnet.sui.io/graphql")]
    rpc: String,

    #[arg(long, default_value_t = Address::from_static(
        "0xf6f30ee0450f6e3e628b68ac473699f26da5063f74be1868155a8a83b8b45060",
    ))]
    ch: Address,

    /// Only the summary of query time and number of positions.
    #[arg(long, short)]
    summary: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc, ch, summary } = Args::parse();
    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());

    tokio::pin!(
        let stream = client.clearing_house_positions(ch, Some(260697587));
    );

    let start = Instant::now();
    let spinner = spinner();
    let mut count = 0;
    while let Some(position) = stream.try_next().await? {
        count += 1;
        if !summary {
            println!("Account id {}", position.0);
            println!("{}", position.1.value);
        } else {
            spinner.tick();
        }
    }
    spinner.finish_using_style();
    println!("Elapsed {:?}", Instant::now().duration_since(start));
    println!("Positions: {count}");
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
