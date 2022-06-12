use exc::{service::ExchangeService, types::trading::Place};
use exc_okx::{
    key::Key,
    websocket::{Endpoint, Request},
};
use rust_decimal_macros::dec;
use std::env::var;
use tower::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "okx_ws_trade=debug,exc_okx=trace".into()),
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
    let place = Place::with_size(dec!(10)).limit(dec!(0.06));
    let req = Request::order("DOGE-USDT", &place);
    let resp = channel.call(req).await?.into_unary()?.await?;
    tracing::info!("resp={resp:?}");
    Ok(())
}
