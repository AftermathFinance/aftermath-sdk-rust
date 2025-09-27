use std::str::FromStr;

use af_ptbuilder::ptb;
use af_sui_types::{ObjectId, TypeTag};
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
    let account_obj_id =
        ObjectId::from_str("0x2b01d94dcb7a9451306223acefd7867968f881d24ce8f9bc564acc767f32e4b7")?;
    // Account's collateral type
    let otw = TypeTag::from_str(
        "0x457049371f5b5dc2bda857bb804ca6e93c5a3cae1636d0cd17bb6b6070d19458::usdc::USDC",
    )?;
    // USDC coin to deposit
    let coin_object_id =
        ObjectId::from_str("0x7fbee197cf2e126c3a64f3a3848fd62daaa329c0619c284ceb0369a471782ce6")?;

    // Fetch the account and the USDC coin object references from the chain using GQL client.
    object_args!({
        account: account_obj_id,
        coin_object: coin_object_id,
    } with { &client });

    // Similar function exists for `SubAccount`, named `deposit_collateral_subaccount`.
    // To use it, you must fetch your subaccount's `ObjectArg` before and pass it instead
    // of `account`
    let ptb = ptb!(
        package perpetuals: perpetuals_package;

        type T = otw.into();

        input obj account;
        input obj coin_object;

        perpetuals::interface::deposit_collateral<T>(account, coin_object);
    );

    println!("PTB: {ptb:?}");

    Ok(())
}
