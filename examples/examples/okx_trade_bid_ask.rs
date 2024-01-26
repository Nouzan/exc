use exc::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,okx_trade_bid_ask=debug,exc_okx=debug".into()),
        ))
        .init();

    let mut okx = Okx::endpoint().connect_exc();
    let mut trades = okx.subscribe_trades("BTC-USDT").await?;
    let mut bid_ask = okx.subscribe_bid_ask("BTC-USDT").await?;
    let jh = tokio::spawn(async move {
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
    jh.await??;
    Ok(())
}
