use std::time::Duration;

use exc::prelude::*;
use futures::StreamExt;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fmt = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,exc_okx=debug,okx_reconnect=debug".into()),
        ));
    tracing_subscriber::registry().with(fmt).init();

    let mut exchange = Okx::endpoint()
        .ws_ping_timeout(Duration::from_secs(5))
        .ws_connection_timeout(Duration::from_secs(5))
        .connect_exc();

    let handles = ["BTC-USDT", "ETH-USDT", "LTC-USDT", "DOGE-USDT"]
        .into_iter()
        .map(|inst| {
            let mut client = exchange.clone();
            tokio::spawn(async move {
                loop {
                    tracing::info!("{inst}");
                    match { client.subscribe_tickers(inst).await } {
                        Ok(mut stream) => {
                            while let Some(c) = stream.next().await {
                                match c {
                                    Ok(c) => tracing::info!("{c}"),
                                    Err(err) => {
                                        tracing::error!("{err}");
                                    }
                                }
                            }
                            tracing::warn!("stream is dead; reconnecting..");
                        }
                        Err(err) => {
                            tracing::error!("request error: {err}; retrying..");
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            })
        })
        .collect::<Vec<_>>();

    tokio::time::sleep(Duration::from_secs(5)).await;
    exchange.reconnect().await?;

    for h in handles {
        let _ = h.await;
    }

    Ok(())
}
