use exc::{
    types::trading::{CancelOrder, Place, PlaceOrder},
    ExchangeLayer,
};
use exc_okx::{key::Key, websocket::Endpoint};
use rust_decimal_macros::dec;
use std::env::var;
use tower::{ServiceBuilder, ServiceExt};

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

    let channel = Endpoint::default()
        .request_timeout(std::time::Duration::from_secs(5))
        .private(key)
        .connect();
    let mut svc = ServiceBuilder::default()
        .layer(ExchangeLayer::default())
        .service(channel);
    let place = Place::with_size(dec!(10)).limit(dec!(0.06));
    let req = PlaceOrder {
        instrument: "DOGE-USDT".to_string(),
        place,
    };
    let id = (&mut svc).oneshot(req).await?.await?;
    tracing::info!("id={id:?}");
    (&mut svc)
        .oneshot(CancelOrder {
            instrument: "DOGE-USDT".to_string(),
            id,
        })
        .await?
        .await?;
    tracing::info!("cancelled");
    Ok(())
}