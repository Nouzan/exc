use exc::prelude::*;
use futures::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,okx_tba_ticker=debug,exc_okx=debug".into()),
        ))
        .init();

    let inst = std::env::var("INST")?;
    let mut okx = Okx::endpoint().connect_exc().into_subscribe_tickers();

    let mut revision = 0;
    loop {
        revision += 1;
        match okx.subscribe_tickers(&inst).await {
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
