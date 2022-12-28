use std::{sync::Arc, time::Duration};

use exc::{
    instrument::service::InstrumentsLayer,
    prelude::*,
    types::instrument::{GetInstrument, InstrumentMeta},
};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stdout)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,exc::instrument=trace,okx_instruments=debug".into()),
        ))
        .init();

    let mut market = Okx::endpoint().connect_exc().layer(
        InstrumentsLayer::new(&["SPOT", "FUTURES", "SWAP"])
            .set_fetch_rate_limit(2, Duration::from_secs(1)),
    );
    for name in ["BTC-USDT", "BTC-USDT-SWAP", "BTC-USD-221230"] {
        let meta: Option<Arc<InstrumentMeta<Decimal>>> = market
            .request(GetInstrument::with_name(name).into())
            .await?
            .try_into()?;
        if let Some(meta) = meta {
            println!("{meta}");
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
        }
    }
    Ok(())
}
