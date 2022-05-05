use exc::transport::http::endpoint::Endpoint;
use exc_okx::http::{
    layer::OkxHttpApiLayer,
    types::request::{history_candles::HistoryCandles, Get, HttpRequest},
};
use tower::{ServiceBuilder, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "okx_http_basic=debug,exc_okx=debug".into()),
        ))
        .init();
    tracing::debug!("hello world");
    let channel = Endpoint::default().connect_https();
    let mut svc = ServiceBuilder::default()
        .layer(OkxHttpApiLayer::default())
        .service(channel);
    let resp = (&mut svc)
        .oneshot(HttpRequest::Get(Get::HistoryCandles(HistoryCandles {
            inst_id: "BTC-USDT".to_string(),
            after: None,
            before: None,
            bar: Some("1m".to_string()),
            limit: None,
        })))
        .await?;
    tracing::info!("{resp:?}");
    Ok(())
}
