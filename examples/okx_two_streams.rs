use exc_okx::websocket::{
    types::{request::Request, response::Response},
    OkxWsClient, WsEndpoint,
};
use futures::StreamExt;

async fn subscribe_tickers(client: &mut OkxWsClient, inst: &str) -> anyhow::Result<Response> {
    Ok(client.send(Request::subscribe_tickers(inst)).await?.await?)
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
    let mut s2 = subscribe_tickers(&mut client, "ETH-USDT")
        .await?
        .into_result()?;
    let mut s1 = subscribe_tickers(&mut client, "BTC-USDT")
        .await?
        .into_result()?;
    let h1 = tokio::spawn(async move {
        while let Some(c) = s1.next().await {
            println!("{c:?}");
        }
    });
    let h2 = tokio::spawn(async move {
        while let Some(c) = s2.next().await {
            println!("{c:?}");
        }
    });
    for h in [h1, h2] {
        let _ = h.await;
    }

    Ok(())
}
