use exc::{
    trading::{CheckOrderService, TradingService},
    transport::http::endpoint::Endpoint as HttpEndpoint,
    types::trading::Place,
    ExchangeLayer,
};
use exc_okx::{
    http::{layer::OkxHttpApiLayer, types::request::HttpRequest},
    key::Key,
    websocket::Endpoint,
};
use rust_decimal_macros::dec;
use std::env::var;
use tower::ServiceBuilder;

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
        .private(key.clone())
        .connect();
    let mut ws = ServiceBuilder::default()
        .layer(ExchangeLayer::default())
        .service(channel);

    let mut http = ServiceBuilder::default()
        .rate_limit(59, std::time::Duration::from_secs(2))
        .layer(ExchangeLayer::<HttpRequest>::default())
        .layer(OkxHttpApiLayer::default().private(key))
        .service(HttpEndpoint::default().connect_https());

    let inst = "DOGE-USDT";
    let id = ws
        .place(inst, &Place::with_size(dec!(-10)).limit(dec!(0.06)))
        .await?;
    tracing::info!("id={id:?}");
    let order = http.check(inst, &id).await?;
    tracing::info!("order={order:?}");
    ws.cancel(inst, &id).await?;
    tracing::info!("cancelled");
    let order = http.check(inst, &id).await?;
    tracing::info!("order={order:?}");
    Ok(())
}
