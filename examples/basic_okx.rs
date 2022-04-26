use exc_okx::websocket::{WsEndpoint, WsRequest};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "basic_okx=debug,exc_okx=debug".into()),
        ))
        .init();

    let mut channel = WsEndpoint::default().connect().await?;
    let req = WsRequest::subscribe_tickers("BTC-USDT");
    let resp = channel.send(req).await?.await?;
    tracing::info!("responsed!");
    let mut stream = resp.into_stream();
    for c in stream.next().await {
        println!("{c:?}");
    }
    Ok(())
}
