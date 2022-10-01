use std::ops::Bound;
use std::time::Duration;

use exc::services::fetch_candles::FetchLastCandlesService;
use exc::types::candle::{Period, QueryCandles};
use exc::Okx;
use futures::StreamExt;
use time::macros::{datetime, offset};
use tower::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,okx_candle=debug,exc_okx=trace".into()),
        ))
        .init();

    let mut svc = Okx::endpoint()
        .connect_exc()
        .into_rate_limited(19, Duration::from_secs(1))
        .into_fetch_candles_backward_with_bound(100, 1);

    let range = (
        Bound::Excluded(datetime!(2020-04-15 00:00:00 +08:00)),
        Bound::Excluded(datetime!(2021-04-15 00:10:00 +08:00)),
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
