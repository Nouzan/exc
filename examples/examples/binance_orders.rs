use clap::Parser;
use exc::{core::types::SubscribeOrders, prelude::*};
use futures::StreamExt;
use std::time::Duration;

#[derive(Parser)]
struct Args {
    inst: String,
    #[clap(long, env)]
    binance_key: String,
    #[clap(long, short, default_value = "12h")]
    reconnect: humantime::Duration,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_orders=debug,exc_binance=debug".into()),
        ))
        .with_line_number(true)
        .init();

    let args = Args::parse();
    let key = serde_json::from_str(&args.binance_key)?;

    let endpoint = std::env::var("ENDPOINT").unwrap_or_else(|_| String::from("binance-u"));
    let mut endpoint = match endpoint.as_str() {
        "binance-u" => Binance::usd_margin_futures(),
        "binance-s" => Binance::spot(),
        _ => anyhow::bail!("unsupported"),
    };

    let binance = endpoint
        .private(key)
        .ws_listen_key_stop_refreshing_after(args.reconnect.into())
        .connect_exc();

    let inst = args.inst.clone();
    let mut market = binance
        .clone()
        .into_subscribe_tickers()
        .into_retry(Duration::from_secs(30));
    tokio::spawn(async move {
        let mut revision = 0;
        loop {
            revision += 1;
            match market.subscribe_tickers(&inst).await {
                Ok(mut stream) => {
                    while let Some(ticker) = stream.next().await {
                        match ticker {
                            Ok(ticker) => {
                                if !ticker.size.is_zero() {
                                    tracing::info!(rev = revision, %inst, "{ticker}");
                                }
                            }
                            Err(err) => {
                                tracing::error!(rev = revision, %inst, "stream error: {err}");
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(
                        rev = revision,
                        %inst,
                        "request new stream error: {err}"
                    );
                }
            }
        }
    });

    let mut revision = 0;
    let mut orders = binance
        .into_adapted::<SubscribeOrders>()
        .into_retry(Duration::from_secs(30));
    loop {
        revision += 1;
        match orders.subscribe_orders(&args.inst).await {
            Ok(mut orders) => {
                while let Some(t) = orders.next().await {
                    match t {
                        Ok(t) => {
                            tracing::info!(rev = revision, "{t:#?}");
                        }
                        Err(err) => {
                            tracing::error!(rev = revision, "stream error: {err}");
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                tracing::error!(rev = revision, "request error: {err}");
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
