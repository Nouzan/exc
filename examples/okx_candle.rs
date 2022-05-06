use exc::{
    transport::http::endpoint::Endpoint,
    types::candle::{Period, QueryCandles},
    ExchangeLayer, FetchCandlesBackwardLayer,
};
use exc_okx::http::layer::OkxHttpApiLayer;
use futures::StreamExt;
use time::macros::{datetime, offset};
use tower::retry::Policy;
use tower::{ServiceBuilder, ServiceExt};

#[derive(Debug, Clone, Copy)]
struct Always;

impl<Req: Clone, Res, E> Policy<Req, Res, E> for Always {
    type Future = futures::future::Ready<Self>;
    fn retry(&self, _req: &Req, result: Result<&Res, &E>) -> Option<Self::Future> {
        if result.is_ok() {
            None
        } else {
            Some(futures::future::ready(Self))
        }
    }

    fn clone_request(&self, req: &Req) -> Option<Req> {
        Some(req.clone())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "okx_candle=debug,exc_okx=debug".into()),
        ))
        .init();
    let channel = ServiceBuilder::default()
        .layer(ExchangeLayer::default())
        .layer(OkxHttpApiLayer::default())
        .service(Endpoint::default().connect_https());
    let mut svc = ServiceBuilder::default()
        .layer(FetchCandlesBackwardLayer::new(100, 1))
        .retry(Always)
        .buffer(2)
        .rate_limit(20, std::time::Duration::from_secs(2))
        .service(channel);

    let query = QueryCandles::new(
        "BTC-USDT",
        Period::minutes(offset!(+8), 1),
        datetime!(2020-04-15 00:00:00 +08:00)..=datetime!(2021-04-16 00:00:00 +08:00),
    );
    let mut stream = (&mut svc).oneshot(query).await?;
    while let Some(c) = stream.next().await {
        match c {
            Ok(c) => tracing::info!("{c}"),
            Err(err) => tracing::error!("{err}"),
        }
    }
    Ok(())
}
