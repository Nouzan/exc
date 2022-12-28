use std::{sync::Arc, time::Duration};

use exc::{
    instrument::service::InstrumentsLayer,
    prelude::*,
    types::instrument::{GetInstrument, InstrumentMeta},
    util::instrument::PollInstrumentsLayer,
    ExcLayer,
};
use rust_decimal::Decimal;
use tower::layer::util::Stack;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stdout)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,exc::market=trace,exc_core::retry=trace".into()),
        ))
        .init();

    let mut market = Binance::usd_margin_futures().connect_exc().layer(
        &InstrumentsLayer::new(&[""]).subscribe_instruments(Stack::new(
            ExcLayer::default(),
            PollInstrumentsLayer::new(Duration::from_secs(60 * 60)),
        )),
    );
    for name in ["btcusdt", "btcusdt_221230"] {
        let meta: Option<Arc<InstrumentMeta<Decimal>>> = market
            .request(GetInstrument::with_name(name).into())
            .await?
            .try_into()?;
        if let Some(meta) = meta {
            println!("{meta}");
        } else {
            println!("`{name}` not found")
        }
    }
    for symbol in ["BTC-USDT", "P:BTC-USDT", "F221230:BTC-USDT"] {
        let symbol = symbol.parse()?;
        let meta: Option<Arc<InstrumentMeta<Decimal>>> = market
            .request(GetInstrument::with_symbol(&symbol).into())
            .await?
            .try_into()?;
        if let Some(meta) = meta {
            println!("{meta}");
        } else {
            println!("`{symbol}` not found")
        }
    }
    Ok(())
}
