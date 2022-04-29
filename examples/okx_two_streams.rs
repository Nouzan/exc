use exc_okx::websocket::{
    types::{request::Request, response::ServerStream},
    Client, Endpoint,
};
use futures::StreamExt;

async fn subscribe_tickers(client: &mut Client, inst: &str) -> anyhow::Result<ServerStream> {
    Ok(client
        .send(Request::subscribe_tickers(inst))
        .await?
        .await?
        .into_result()?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "basic_okx=debug,okx_two_streams=debug".into()),
        ))
        .init();

    let client = Endpoint::default().connect();
    let handles = ["BTC-USDT", "ETH-USDT"]
        .into_iter()
        .map(|inst| {
            let mut shared = client.clone();
            tokio::spawn(async move {
                loop {
                    tracing::info!("{inst}");
                    match { subscribe_tickers(&mut shared, inst).await } {
                        Ok(mut stream) => {
                            let mut count = 0;
                            while let Some(c) = stream.next().await {
                                tracing::info!("{c:?}");
                                count += 1;
                                if count > 10 {
                                    break;
                                }
                            }
                            tracing::warn!("stream is dead; reconnecting..");
                        }
                        Err(err) => {
                            tracing::error!("request error: {err}; retrying..");
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            })
        })
        .collect::<Vec<_>>();
    for h in handles {
        let _ = h.await;
    }

    Ok(())
}
