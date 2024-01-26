use std::time::Duration;

use exc::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_reconnect=debug,exc_binance=debug".into()),
        ))
        .init();

    let mut endpoint = match std::env::var("ENDPOINT")?.as_str() {
        "binance-u" => Binance::usd_margin_futures(),
        "binance-s" => Binance::spot(),
        other => anyhow::bail!("unsupported endpoint {other}"),
    };
    let mut exc = endpoint
        .ws_rate_limit(2, Duration::from_secs(1))
        .connect_exc();

    let handles = ["bnbusdt", "ltcusdt", "btcusdt", "ethusdt"]
        .into_iter()
        .map(|inst| {
            let mut client = exc
                .clone()
                .into_subscribe_tickers()
                .into_retry(Duration::from_secs(30));

            tokio::spawn(async move {
                loop {
                    tracing::info!("{inst}");
                    match { client.subscribe_tickers(inst).await } {
                        Ok(mut stream) => {
                            while let Some(c) = stream.next().await {
                                match c {
                                    Ok(c) => tracing::info!("{inst}; {c}"),
                                    Err(err) => {
                                        tracing::error!("{inst}; {err}");
                                    }
                                }
                            }
                            tracing::warn!("{inst} stream is dead; reconnecting..");
                        }
                        Err(err) => {
                            tracing::error!("{inst} request error: {err}; retrying..");
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            })
        })
        .collect::<Vec<_>>();

    tokio::time::sleep(Duration::from_secs(5)).await;
    exc.reconnect().await?;

    for h in handles {
        let _ = h.await;
    }

    Ok(())
}
