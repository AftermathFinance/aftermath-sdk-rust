use std::time::Instant;

use clap::Parser;
use color_eyre::Result;
use futures::TryStreamExt as _;
use indicatif::ProgressBar;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://graphql.testnet.sui.io/graphql")]
    rpc: String,

    /// Only the summary of query time and number of objects.
    #[arg(long, short)]
    summary: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc, summary } = Args::parse();
    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());
    let owner = None;
    let type_ = Some(
        "0x1164da999906ab20f4a82be965bdd5e505367edaf55e94e5dc7b29228bf5d88a::account::AccountCap<0x457049371f5b5dc2bda857bb804ca6e93c5a3cae1636d0cd17bb6b6070d19458::usdc::USDC>".into(),
    );

    tokio::pin!(
        let stream = client.latest_full_objects(owner, type_, None);
    );

    let start = Instant::now();
    let spinner = spinner();
    let mut count = 0;
    while let Some(obj) = stream.try_next().await? {
        count += 1;
        if summary {
            spinner.tick();
        } else {
            println!("Object ID: {:?}", obj.object_id());
            println!("Object: {obj:?}");
            println!("Object Owner: {:?}", obj.owner());
        }
    }
    spinner.finish_using_style();
    println!("Elapsed: {:?}", Instant::now().duration_since(start));
    println!("Objects count: {count}");
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
