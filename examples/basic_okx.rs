use exc_okx::websocket::{
    types::{request::Request, response::Response},
    OkxWsClient, WsEndpoint,
};
use futures::StreamExt;

async fn request(client: &mut OkxWsClient, req: Request) -> anyhow::Result<Response> {
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
        let req = Request::subscribe_tickers("ETH-USDT");
        match request(&mut client, req).await {
            Ok(resp) => {
                match resp.into_result() {
                    Ok(mut stream) => {
                        let mut count = 0;
                        while let Some(c) = stream.next().await {
                            println!("{c:?}");
                            count += 1;
                            if count > 10 {
                                break;
                            }
                        }
                        tracing::info!("stream is dead; reconnecting...");
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
