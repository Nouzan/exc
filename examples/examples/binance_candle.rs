use exc::prelude::*;
use futures::StreamExt;
use std::time::Duration;
use time::{macros::datetime, UtcOffset};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_candle=debug,exc_binance=debug".into()),
        ))
        .init();

    let endpoint = std::env::var("ENDPOINT").unwrap_or_else(|_| String::from("binance-u"));
    let endpoint = match endpoint.as_str() {
        "binance-u" => Binance::usd_margin_futures(),
        "binance-s" => Binance::spot(),
        "binance-e" => Binance::european_options(),
        _ => anyhow::bail!("unsupported"),
    };

    let inst = std::env::var("INST").unwrap_or_else(|_| String::from("btcusdt"));

    let mut binance = endpoint
        .connect_exc()
        .into_rate_limited(200, Duration::from_secs(60))
        .into_fetch_candles_forward(1000);
    let mut stream = binance
        .fetch_candles_range(
            &inst,
            Period::minutes(UtcOffset::UTC, 1),
            datetime!(2020-06-27 00:00:00 +08:00)..,
        )
        .await?;
    while let Some(candle) = stream.next().await {
        let candle = candle?;
        tracing::info!("{candle}");
    }
    Ok(())
}
