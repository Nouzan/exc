use exc_okx::websocket::{WsChannel, WsEndpoint, WsRequest};

async fn request(channel: &mut WsChannel, req: WsRequest) -> anyhow::Result<()> {
    let resp = channel.send(req).await?.await?;
    tracing::info!("{resp:?}");
    Ok(())
}

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
    if let Err(err) = request(&mut channel, req).await {
        tracing::error!("{err}");
    }
    let req = WsRequest::unsubscribe_tickers("BTC-USDT");
    if let Err(err) = request(&mut channel, req).await {
        tracing::error!("{err}");
    }
    tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    Ok(())
}
