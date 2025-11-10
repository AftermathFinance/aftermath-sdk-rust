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
    let owner = "0xb02cad883811f506630fb6dee187496b075b1c3fed62b237a44af9665be5df66".parse()?;
    let coin_type = Some("0x2::coin::Coin<0x2::sui::SUI>".into());

    tokio::pin!(
        let stream = client.owner_gas_coins(owner, coin_type, None);
    );

    let start = Instant::now();
    let spinner = spinner();
    let mut count = 0;
    while let Some((ckpt_num, obj_ref, balance)) = stream.try_next().await? {
        count += 1;
        if summary {
            spinner.tick();
        } else {
            println!("Ckpt num: {:?}", ckpt_num);
            println!("Object Ref: {:?}", obj_ref);
            println!("Coin balance: {:?}", balance);
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
