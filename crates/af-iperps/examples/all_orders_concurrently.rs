use std::collections::BTreeMap;
use std::fmt::Display;
use std::time::Instant;

use af_iperps::graphql::{GraphQlClientExt as _, OrderMaps};
use af_iperps::order_helpers::Side;
use af_iperps::order_id::{order_side, price_ask, price_bid};
use af_iperps::ClearingHouse;
use af_move_type::MoveInstance;
use af_sui_types::{hex_address_bytes, ObjectId};
use clap::Parser;
use color_eyre::eyre::OptionExt as _;
use color_eyre::Result;
use futures::{stream_select, TryStreamExt as _};
use nonempty::NonEmpty;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;
use textplots::{Chart, ColorPlot as _, Shape};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,

    #[arg(long, default_value_t = ObjectId::new(hex_address_bytes(
        b"0x49bd40cc7880bd358465116157f0271c25d23361b94eace9a25dc2019b449bfc",
    )))]
    ch: ObjectId,

    #[arg(long, default_value_t = 0.005)]
    max_spread: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args {
        rpc,
        ch,
        max_spread,
    } = Args::parse();
    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());

    let ch_obj = client.full_object(ch, None).await?;
    let ch_struct = ch_obj.as_move().ok_or_eyre("Not a Move struct")?;
    let OrderMaps { asks, bids, .. } = client.order_maps(ch_struct.type_.address(), ch).await?;
    let ch_inst = MoveInstance::<ClearingHouse>::from_raw_struct(
        ch_struct.type_.clone().into(),
        &ch_struct.contents,
    )?;

    tokio::pin!(
        let asks_stream = client.map_orders(asks, Some(ch_struct.version));
        let bids_stream = client.map_orders(bids, Some(ch_struct.version));
    );
    let mut stream = stream_select!(asks_stream, bids_stream);

    let mut asks = BTreeMap::new();
    let mut bids = BTreeMap::new();
    let start = Instant::now();
    let spinner = spinner();
    while let Some((id, order)) = stream.try_next().await? {
        spinner.tick();
        match order_side(id) {
            Side::Ask => *asks.entry(id).or_insert(0u64) += order.size,
            Side::Bid => *bids.entry(id).or_insert(0u64) += order.size,
        };
    }
    spinner.finish_using_style();
    println!("Elapsed: {:?}", Instant::now().duration_since(start));
    println!("Orders: {}", asks.len() + bids.len());
    println!("Bids: {}", bids.len());
    println!("Asks: {}", asks.len());

    maybe_plot(&ch_inst.value, max_spread, asks, bids);

    Ok(())
}

fn maybe_plot(
    ch: &ClearingHouse,
    max_spread: f64,
    asks: BTreeMap<u128, u64>,
    bids: BTreeMap<u128, u64>,
) {
    let Some(bids) = NonEmpty::from_vec(bids.into_iter().collect()) else {
        return;
    };
    let Some(asks) = NonEmpty::from_vec(asks.into_iter().collect()) else {
        return;
    };

    println!("Min bid {}", obook_price(ch, price_bid(bids.last().0)));
    println!("Max ask {}", obook_price(ch, price_ask(asks.last().0)));

    let mid_price: f64 = obook_price(
        ch,
        (price_bid(bids.first().0) + price_ask(asks.first().0)) / 2,
    )
    .into();
    println!("Mid price {mid_price}");

    let min_price = mid_price * (1.0 - max_spread);
    let max_price = mid_price * (1.0 + max_spread);

    let mut cum_size = 0;
    let bids: Vec<_> = bids
        .into_iter()
        .map_while(|(id, size)| {
            let price: f64 = obook_price(ch, price_bid(id)).into();
            if price < min_price {
                None
            } else {
                cum_size += size;
                Some((price as f32, cum_size as f32))
            }
        })
        .collect();

    cum_size = 0;
    let asks: Vec<_> = asks
        .into_iter()
        .map_while(|(id, size)| {
            let price: f64 = obook_price(ch, price_ask(id)).into();
            if price > max_price {
                None
            } else {
                cum_size += size;
                Some((price as f32, cum_size as f32))
            }
        })
        .collect();

    let green = rgb::Rgb::new(0, 255, 0);
    let red = rgb::Rgb::new(255, 0, 0);
    Chart::new(180, 60, min_price as f32, max_price as f32)
        .linecolorplot(&Shape::Steps(&bids), green)
        .linecolorplot(&Shape::Steps(&asks), red)
        .display();
}

const fn obook_price(ch: &ClearingHouse, price: u64) -> TicksPerLot {
    TicksPerLot {
        price,
        lot_size: ch.market_params.lot_size,
        tick_size: ch.market_params.tick_size,
    }
}

#[derive(Debug, Clone, Copy)]
struct TicksPerLot {
    price: u64,
    lot_size: u64,
    tick_size: u64,
}

impl From<TicksPerLot> for f64 {
    fn from(value: TicksPerLot) -> Self {
        let TicksPerLot {
            price,
            lot_size,
            tick_size,
        } = value;
        (price as Self * tick_size as Self) / lot_size as Self
    }
}

impl Display for TicksPerLot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f64::from(*self).fmt(f)
    }
}

// https://github.com/console-rs/indicatif/blob/main/examples/long-spinner.rs
fn spinner() -> indicatif::ProgressBar {
    use indicatif::{ProgressFinish, ProgressStyle};
    let pb = indicatif::ProgressBar::new_spinner();
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
