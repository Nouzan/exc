use exc::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "error,binance_fetch_instruments=debug,exc_binance=debug".into()
            }),
        ))
        .init();

    let endpoint = std::env::var("ENDPOINT").unwrap_or_else(|_| String::from("binance-u"));
    let endpoint = match endpoint.as_str() {
        "binance-u" => Binance::usd_margin_futures(),
        "binance-s" => Binance::spot(),
        "binance-e" => Binance::european_options(),
        _ => anyhow::bail!("unsupported"),
    };

    let mut binance = endpoint.connect_exc();
    let mut stream = binance.fetch_instruments("").await?;
    while let Some(meta) = stream.next().await {
        let meta = meta?;
        tracing::info!("{meta}");
    }
    Ok(())
}
