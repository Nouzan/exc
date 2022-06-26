use exc::transport::http::endpoint::Endpoint;
use exc_binance::http::{
    layer::BinanceRestApiLayer,
    request::{utils::Ping, RestRequest},
    response::Data,
};
use tower::{ServiceBuilder, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,exc_binance=trace,binance_http_ping=debug".into()),
        ))
        .init();

    let http = Endpoint::default().connect_https();
    let svc = ServiceBuilder::default()
        .layer(BinanceRestApiLayer)
        .service(http);
    let resp: Data = svc
        .oneshot(RestRequest::from(Ping::default()))
        .await?
        .into_inner();
    tracing::info!("{resp:?}");
    Ok(())
}
