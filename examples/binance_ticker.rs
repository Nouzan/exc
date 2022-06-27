use exc::service::{
    subscribe_tickers::{SubscribeTickersService, TradeBidAskService},
    ExchangeService,
};
use exc_binance::Binance;
use futures::StreamExt;

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
    let mut tickers = binance.subscribe_tickers("btcbusd").await?;
    while let Some(t) = tickers.next().await {
        let t = t?;
        tracing::info!("{t}");
    }
    Ok(())
}
