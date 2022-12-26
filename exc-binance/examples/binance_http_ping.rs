use std::time::{Duration, Instant};

use exc::core::transport::http::endpoint::Endpoint;
use exc_binance::http::{
    layer::BinanceRestApiLayer,
    request::{utils::Ping, RestRequest},
    response::Data,
};
use humantime::format_duration;
use tower::{ServiceBuilder, ServiceExt};

struct Stats {
    count: u32,
    total: Duration,
    max: Option<Duration>,
    min: Option<Duration>,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            count: 0,
            total: Duration::ZERO,
            max: None,
            min: None,
        }
    }
}

impl Stats {
    fn update(&mut self, rtt: Duration) {
        self.count += 1;
        self.total += rtt;
        self.max = Some(self.max.map(|max| max.max(rtt)).unwrap_or(rtt));
        self.min = Some(self.min.map(|min| min.min(rtt)).unwrap_or(rtt));
    }

    fn show(&self) {
        if self.count > 0 {
            println!(
                "max={} min={} avg={}",
                format_duration(self.max.unwrap()),
                format_duration(self.min.unwrap()),
                format_duration(self.total / self.count),
            );
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,exc_binance=trace,binance_http_ping=trace".into()),
        ))
        .init();
    let times: usize = std::env::var("TIMES")
        .ok()
        .and_then(|times| times.parse().ok())
        .unwrap_or(100);
    let http = Endpoint::default().connect_https();
    let mut svc = ServiceBuilder::default()
        .layer(BinanceRestApiLayer::usd_margin_futures())
        .service(http);
    let mut stats = Stats::default();
    for _ in 0..times {
        let begin = Instant::now();
        let resp: Data = (&mut svc)
            .oneshot(RestRequest::with_payload(Ping))
            .await?
            .into_inner();
        let end = Instant::now();
        let rtt = end.duration_since(begin);
        tracing::trace!("{resp:?}");
        tracing::info!("rtt={}", format_duration(rtt));
        stats.update(rtt);
    }
    stats.show();
    Ok(())
}
