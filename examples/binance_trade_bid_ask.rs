use exc::{book::SubscribeBidAskService, trade::SubscribeTradesService, ExchangeService};
use exc_binance::Binance;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_trade=debug,exc_binance=debug".into()),
        ))
        .init();

    let mut binance = Binance::usd_margin_futures().connect().into_exchange();
    let mut trades = binance.subscribe_trades("btcbusd").await?;
    let mut bid_ask = binance.subscribe_bid_ask("btcbusd").await?;
    tokio::spawn(async move {
        while let Some(t) = bid_ask.next().await {
            let t = t?;
            tracing::info!("bid-ask: {t}");
        }
        Result::<_, anyhow::Error>::Ok(())
    });
    while let Some(t) = trades.next().await {
        let t = t?;
        tracing::info!("trade: {t}");
    }
    Ok(())
}
