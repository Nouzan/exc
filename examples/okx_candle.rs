use exc::{
    transport::http::endpoint::Endpoint,
    types::candle::{Period, QueryLastCandles},
    Exchange,
};
use exc_okx::http::{layer::OkxHttpApiLayer, types::request::HttpRequest};
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
    tracing::debug!("hello world");
    let channel = Endpoint::default().connect_https();
    let svc = ServiceBuilder::default()
        .layer(OkxHttpApiLayer::default())
        .service(channel);
    let mut ex = Exchange::<_, HttpRequest>::new(svc);
    let query = QueryLastCandles::new(
        "BTC-USDT",
        Period::minutes(offset!(+8), 1),
        datetime!(2020-01-01 00:00:00 +08:00)..=datetime!(2020-01-01 00:02:00 +08),
        2,
    );
    let mut stream = (&mut ex).oneshot(query).await?;
    while let Some(Ok(c)) = stream.next().await {
        tracing::info!("{c}");
    }
    Ok(())
}
