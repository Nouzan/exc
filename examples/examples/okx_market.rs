use std::{sync::Arc, time::Duration};

use exc::{
    market::{request::Request, service::MarketLayer},
    prelude::*,
    types::instrument::{GetInstrument, InstrumentMeta},
};
use rust_decimal::Decimal;
use tower::{Service, ServiceExt};

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
    // market.ready().await?;
    market
        .request(GetInstrument::with_name("BTC-USDT").into())
        .await?;
    tokio::time::sleep(Duration::from_secs(10)).await;
    let meta: Option<Arc<InstrumentMeta<Decimal>>> = market
        .request(GetInstrument::with_name("BTC-USDT").into())
        .await?
        .try_into()?;
    if let Some(meta) = meta {
        println!("{meta}");
    }
    Ok(())
}
