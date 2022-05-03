use exc::Exchange;
use exc_okx::websocket::{types::request::Request, Endpoint};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "basic_okx=debug,exc_okx=debug".into()),
        ))
        .init();

    let channel = Endpoint::default()
        .timeout(std::time::Duration::from_secs(5))
        .connect();
    let mut client = Exchange::new(channel);
    loop {
        let req = Request::subscribe_tickers("ETH-USDT");
        match client.request(req).await {
            Ok(resp) => {
                tracing::info!("responsed");
                match resp.into_result() {
                    Ok(mut stream) => {
                        let mut count = 0;
                        while let Some(c) = stream.next().await {
                            tracing::info!("{c:?}");
                            count += 1;
                            if count > 10 {
                                break;
                            }
                        }
                        tracing::warn!("stream is dead; reconnecting...");
                    }
                    Err(status) => {
                        tracing::error!("request error: {}; retrying...", status);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(err) => {
                tracing::error!("transport error: {err}; retrying...");
            }
        }
    }
}
