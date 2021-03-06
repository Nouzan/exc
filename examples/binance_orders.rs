use clap::Parser;
use exc::{IntoExc, SubscribeOrdersService};
use exc_binance::Binance;
use futures::StreamExt;
use std::time::Duration;

#[derive(Parser)]
struct Args {
    #[clap(long, env)]
    binance_key: String,
    inst: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_orders=debug,exc_binance=debug".into()),
        ))
        .init();

    let args = Args::from_args();
    let key = serde_json::from_str(&args.binance_key)?;

    let mut binance = Binance::usd_margin_futures()
        .private(key)
        .connect()
        .into_exc();

    let mut revision = 0;
    loop {
        revision += 1;
        match binance.subscribe_orders(&args.inst).await {
            Ok(mut orders) => {
                while let Some(t) = orders.next().await {
                    match t {
                        Ok(t) => {
                            tracing::info!("[{revision}]{t:#?}");
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
