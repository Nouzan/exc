use exc::prelude::*;
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
    // For options or other contracts, you can use query string to provide with uly: "OPTION?uly=BTC-USD"
    let tag = std::env::var("INST_TAG").unwrap_or("SWAP".to_string());
    let mut api = Okx::endpoint().connect_exc();
    let mut stream = api.fetch_instruments(&tag).await?;
    while let Some(meta) = stream.next().await {
        let meta = meta?;
        tracing::info!("{meta}");
    }
    Ok(())
}
