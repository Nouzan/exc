use std::env;

use exc_okx::{
    websocket::types::messages::{event::OkxOptionSummary, Args},
    Okx, OkxRequest,
};
use futures::StreamExt;
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
    let mut okx = Okx::endpoint().connect();
    let mut stream = okx
        .ready()
        .await?
        .call(OkxRequest::subscribe(Args::subscribe_option_summary(
            &inst_family,
        )))
        .await?
        .ws()?
        .into_stream()?;
    while let Some(frame) = stream.next().await.transpose()? {
        let Some(datas) = frame.to_deserialized_changes::<OkxOptionSummary>() else {
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
