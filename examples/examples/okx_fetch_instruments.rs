use exc::{FetchInstrumentsService, Okx};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,okx_fetch_instruments=debug,exc_okx=debug".into()),
        ))
        .init();

    let mut api = Okx::endpoint().connect_exc();
    let mut stream = api.fetch_instruments("FUTURES").await?;
    while let Some(meta) = stream.next().await {
        let meta = meta?;
        tracing::info!("{meta}");
    }
    Ok(())
}
