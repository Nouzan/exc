use exc::{ExchangeLayer, SubscribeTickersService};
use exc_okx::websocket::Endpoint;
use futures::StreamExt;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "exc_okx=debug,okx_two_streams=debug".into()),
        ))
        .init();

    let endpoint = Endpoint::default()
        .timeout(std::time::Duration::from_secs(5))
        .connection_timeout(std::time::Duration::from_secs(5));
    let exchange = ServiceBuilder::new()
        .layer(ExchangeLayer::default())
        .service(endpoint.connect());

    let handles = ["BTC-USDT", "ETH-USDT", "LTC-USDT", "DOGE-USDT"]
        .into_iter()
        .map(|inst| {
            let mut client = exchange.clone();
            tokio::spawn(async move {
                loop {
                    tracing::info!("{inst}");
                    match { client.subscribe_tickers(inst).await } {
                        Ok(mut stream) => {
                            while let Some(c) = stream.next().await {
                                match c {
                                    Ok(c) => tracing::info!("{c}"),
                                    Err(err) => {
                                        tracing::error!("{err}");
                                    }
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
