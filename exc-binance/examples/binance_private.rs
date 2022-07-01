use std::time::Duration;

use clap::Parser;
use exc_binance::{
    websocket::protocol::frame::{account::AccountEvent, Name},
    Binance, Request,
};
use futures::{pin_mut, StreamExt};
use tower::{Service, ServiceExt};

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
                .unwrap_or_else(|_| "error,binance_private=debug,exc_binance=trace".into()),
        ))
        .init();

    let args = Args::from_args();
    let key = serde_json::from_str(&args.binance_key)?;

    let mut api = Binance::usd_margin_futures()
        .ws_keep_alive_timeout(Duration::from_secs(30))
        .private(key)
        .ws_listen_key_retry(5)
        .ws_listen_key_refresh_interval(Duration::from_secs(60))
        .connect();
    api.ready().await?;
    let stream = api
        .call(Request::subcribe_main(Name::order_trade_update(&args.inst)))
        .await?
        .into_stream::<AccountEvent>()?;
    pin_mut!(stream);
    while let Some(e) = stream.next().await {
        let e = e?;
        tracing::info!("{e:#?}");
    }
    Ok(())
}
