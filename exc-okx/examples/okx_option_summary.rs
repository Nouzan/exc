use std::env;

use exc_okx::{websocket::types::messages::Args, Okx, OkxRequest};
use futures::StreamExt;
use serde_json::Value;
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let inst_family = env::var("INST_FAMILY")?;
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,exc_okx=debug,okx_option_summary=info".into()),
        ))
        .init();
    let mut okx = Okx::endpoint().aws(true).connect();
    let mut stream = okx
        .ready()
        .await?
        .call(OkxRequest::subscribe(Args::subscribe_channel(
            "opt-summary",
            [("instFamily", inst_family.as_str())],
        )))
        .await?
        .ws()?
        .into_stream()?;
    while let Some(frame) = stream.next().await.transpose()? {
        let Some(datas) = frame.to_deserialized_changes::<Value>() else {
            tracing::warn!("not a change: {frame:?}");
            continue;
        };
        for data in datas {
            let order = data?;
            tracing::info!("{order:#?}");
        }
    }
    Ok(())
}
