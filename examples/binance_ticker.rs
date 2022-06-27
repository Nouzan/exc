use exc::service::{
    subscribe_tickers::{SubscribeTickersService, TradeBidAskService},
    ExchangeService,
};
use exc_binance::Binance;
use futures::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_ticker=debug,exc_binance=debug".into()),
        ))
        .init();

    let mut binance = Binance::usd_margin_futures()
        .connect()
        .into_exchange()
        .into_subscribe_tickers();

    let mut revision = 0;
    loop {
        revision += 1;
        match binance.subscribe_tickers("btcbusd").await {
            Ok(mut tickers) => {
                while let Some(t) = tickers.next().await {
                    match t {
                        Ok(t) => {
                            tracing::info!("[{revision}]{t}");
                        }
                        Err(err) => {
                            tracing::error!("[{revision}]stream error: {err}");
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                tracing::error!("[{revision}]request error: {err}");
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
