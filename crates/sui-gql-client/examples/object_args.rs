use clap::Parser;
use color_eyre::Result;
use sui_gql_client::object_args;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc } = Args::parse();
    let client = ReqwestClient::new_default(rpc);
    object_args!({
        mut clearing_house: "0xe4a1c0bfc53a7c2941a433a9a681c942327278b402878e0c45280eecd098c3d1".parse()?,
        registry: "0x400e84251a6ce2192f69c1aa775d68bab7690e059578317bf9e844d40e07e04d".parse()?,
    } with { &client } paged by 10);
    println!("{clearing_house:?}");
    println!("{registry:?}");

    object_args!({
        mut ch: "0xe4a1c0bfc53a7c2941a433a9a681c942327278b402878e0c45280eecd098c3d1".parse()?,
    } with { &client });
    println!("{ch:?}");
    Ok(())
}
