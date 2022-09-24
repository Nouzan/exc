use exc::okx::{
    key::Key,
    websocket::{Endpoint, Request},
};
use std::env::var;
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,okx_ws_login=debug,exc_okx=trace".into()),
        ))
        .init();

    let key = Key {
        apikey: var("OKX_APIKEY")?,
        secretkey: var("OKX_SECRETKEY")?,
        passphrase: var("OKX_PASSPHRASE")?,
    };

    let mut channel = Endpoint::default()
        .request_timeout(std::time::Duration::from_secs(5))
        .private(key)
        .connect();
    channel.ready().await?;
    let req = Request::subscribe_tickers("ETH-USDT");
    channel.call(req).await?;
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    Ok(())
}
