use exc_okx::websocket::{OkxWsClient, WsEndpoint, WsRequest, WsResponse};
use futures::StreamExt;

async fn request(client: &mut OkxWsClient, req: WsRequest) -> anyhow::Result<WsResponse> {
    Ok(client.send(req).await?.await?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "basic_okx=debug,exc_okx=debug".into()),
        ))
        .init();

    let mut client = WsEndpoint::default().connect();
    loop {
        let req = WsRequest::subscribe_tickers("ETH-USDT");
        match request(&mut client, req).await {
            Ok(resp) => {
                let mut stream = resp.into_stream();
                while let Some(c) = stream.next().await {
                    println!("{c:?}");
                }
                tracing::info!("stream is dead; reconnecting...");
            }
            Err(err) => {
                tracing::error!("request error: {err}; retrying...");
            }
        }
    }
}
