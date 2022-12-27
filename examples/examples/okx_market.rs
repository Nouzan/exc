use std::sync::Arc;

use exc::{
    market::service::MarketLayer,
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
                .unwrap_or_else(|_| "error,exc::market=trace,market=debug".into()),
        ))
        .init();

    let mut market = Okx::endpoint().connect_exc().layer(&MarketLayer::default());
    let meta: Option<Arc<InstrumentMeta<Decimal>>> = market
        .request(GetInstrument::with_name("BTC-USDT").into())
        .await?
        .try_into()?;
    if let Some(meta) = meta {
        println!("{meta}");
    }
    Ok(())
}
