use std::str::FromStr;

use af_ptbuilder::ptb;
use af_sui_types::{Address, ObjectId, TypeTag};
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

    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());

    let perpetuals_package =
        ObjectId::from_str("0x3bee2aefb42092215d5a22808e2a0abd49f58ca5a8ecfee9634c406370890233")?;
    let perpetuals_registry =
        ObjectId::from_str("0x8ae8b5a9517644be751ecf10aa83c10fe3dfe32849b7d682c570d7a8215cd942")?;
    // Account's collateral type
    let otw = TypeTag::from_str(
        &"0x457049371f5b5dc2bda857bb804ca6e93c5a3cae1636d0cd17bb6b6070d19458::usdc::USDC",
    )?;
    // Address that is going to receive the created account
    let recipient =
        Address::from_str("0x5958f891c49fb2d2906e6a3f0aa4a5a70634b791dcdbc774c0bef9abd92d3f80")?;

    // Fetch the object reference for the perpetuals registry from the chain using
    // GQL client.
    object_args!({
        mut registry: perpetuals_registry
    } with { &client });

    let ptb = ptb!(
        package perpetuals: perpetuals_package;

        type T = otw.into();

        input obj registry;
        input pure to: &recipient;

        let account = perpetuals::interface::create_account<T>(registry);
        command! TransferObjects(vec![account], to);
    );

    println!("PTB: {ptb:?}");

    Ok(())
}
