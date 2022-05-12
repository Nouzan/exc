use std::ops::Bound;

use exc::{
    transport::http::endpoint::Endpoint,
    types::candle::{Period, QueryCandles},
    ExchangeLayer, FetchCandlesBackwardLayer,
};
use exc_okx::http::layer::OkxHttpApiLayer;
use futures::StreamExt;
use time::macros::{datetime, offset};
use tower::{ServiceBuilder, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "okx_candle=debug,exc_okx=debug".into()),
        ))
        .init();
    let mut svc = ServiceBuilder::default()
        .layer(FetchCandlesBackwardLayer::new(100, 1))
        .rate_limit(19, std::time::Duration::from_secs(2))
        .layer(ExchangeLayer::default())
        .layer(OkxHttpApiLayer::default().retry_on_error())
        .service(Endpoint::default().connect_https());

    let range = (
        Bound::Excluded(datetime!(2020-04-15 00:00:00 +08:00)),
        Bound::Excluded(datetime!(2020-04-15 00:10:00 +08:00)),
    );
    let query = QueryCandles::new("BTC-USDT", Period::minutes(offset!(+8), 1), range);
    let mut stream = (&mut svc).oneshot(query).await?;
    while let Some(c) = stream.next().await {
        match c {
            Ok(c) => tracing::info!("{c}"),
            Err(err) => tracing::error!("{err}"),
        }
    }
    Ok(())
}
