use exc::okx::{key::OkxKey, websocket::Request, Okx};
use exc_okx::OkxRequest;
use std::{env::var, time::Duration};
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,okx_login=debug,exc_okx=trace".into()),
        ))
        .init();

    let key = OkxKey {
        apikey: var("OKX_APIKEY")?,
        secretkey: var("OKX_SECRETKEY")?,
        passphrase: var("OKX_PASSPHRASE")?,
    };
    let mut okx = Okx::endpoint()
        .private(key)
        .ws_request_timeout(Duration::from_secs(5))
        .connect();
    okx.ready().await?;
    let req = Request::subscribe_tickers("ETH-USDT");
    okx.call(OkxRequest::Ws(req)).await?;
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    Ok(())
}
