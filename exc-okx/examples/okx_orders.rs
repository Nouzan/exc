use clap::Parser;
use exc_okx::{websocket::types::messages::event::order::OkxOrder, Okx, OkxRequest};
use futures::StreamExt;
use tower::{Service, ServiceExt};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, env)]
    okx_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;
    Registry::default()
        .with(fmt::layer().with_line_number(true))
        .with(
            EnvFilter::builder()
                .with_default_directive("info".parse()?)
                .from_env_lossy(),
        )
        .init();
    let key = serde_json::from_str(&cli.okx_key)?;
    let mut okx = Okx::endpoint().private(key).connect();
    okx.ready().await?;
    let mut stream = okx
        .call(OkxRequest::subscribe_orders("DOGE-USDT"))
        .await?
        .ws()?
        .into_result()?
        .skip(1);
    while let Some(frame) = stream.next().await.transpose()? {
        for data in frame.into_deserialized_changes::<OkxOrder>().unwrap() {
            let order = data?;
            tracing::debug!("{order:#?}");
        }
    }
    Ok(())
}
