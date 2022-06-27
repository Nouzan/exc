use std::time::Duration;

use exc::{instrument::SubscribeInstrumentsService, ExchangeLayer};
use exc_okx::websocket::{Endpoint, Request};
use futures::StreamExt;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "tower=trace,info".into()),
        ))
        .init();
    let svc = ServiceBuilder::default()
        .layer(ExchangeLayer::<Request>::default())
        .service(
            Endpoint::default()
                .request_timeout(Duration::from_secs(5))
                .connection_timeout(Duration::from_secs(5))
                .connect(),
        );
    let handles = ["SPOT", "FUTURES", "SWAP"]
        .iter()
        .map(|tag| {
            let tag = tag.to_string();
            let mut svc = svc.clone();
            tokio::spawn(async move {
                loop {
                    match svc.subscribe_instruments(&tag).await {
                        Ok(mut stream) => {
                            while let Some(meta) = stream.next().await {
                                match meta {
                                    Ok(meta) => {
                                        tracing::info!("{meta}");
                                    }
                                    Err(err) => {
                                        tracing::error!("stream error: {err}");
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            tracing::error!("request error: {err}");
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            })
        })
        .collect::<Vec<_>>();

    for h in handles {
        h.await?;
    }

    Ok(())
}
