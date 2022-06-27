use exc::{service::fetch_candles::FetchFirstCandlesService, types::Period, FetchCandlesService};
use exc_binance::Binance;
use futures::StreamExt;
use time::{macros::datetime, UtcOffset};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_instrument=debug,exc_binance=debug".into()),
        ))
        .init();

    let mut binance = Binance::usd_margin_futures()
        .connect()
        .into_exchange()
        .into_fetch_candles_forward(100);
    let mut stream = binance
        .fetch_candles(
            "btcbusd",
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
