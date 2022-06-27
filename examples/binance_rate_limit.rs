use std::time::Duration;

use exc::{
    service::fetch_candles::FetchFirstCandlesService, types::Period, ExchangeService,
    FetchCandlesService,
};
use exc_binance::Binance;
use futures::StreamExt;
use time::{macros::datetime, UtcOffset};

async fn task(inst: &str) -> anyhow::Result<()> {
    let mut binance = Binance::usd_margin_futures()
        .connect()
        .into_exchange()
        .into_rate_limited(70, Duration::from_secs(60))
        .into_fetch_candles_forward(1500);
    let mut stream = binance
        .fetch_candles(
            inst,
            Period::minutes(UtcOffset::UTC, 1),
            datetime!(2020-01-01 00:00:00 +08:00)..,
        )
        .await?;
    while let Some(candle) = stream.next().await {
        let candle = candle?;
        tracing::info!("{inst}: {candle}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_rate_limit=debug,exc_binance=debug".into()),
        ))
        .init();

    let handles = [
        "btcusdt", "btcbusd", "ethusdt", "ethbusd", "busdusdt", "dogeusdt", "dogebusd",
    ]
    .into_iter()
    .map(|inst| {
        let inst = inst.to_string();
        tokio::spawn(async move {
            if let Err(err) = task(&inst).await {
                tracing::error!("{inst}: {err}");
            }
        })
    })
    .collect::<Vec<_>>();

    for h in handles {
        h.await?;
    }

    Ok(())
}
