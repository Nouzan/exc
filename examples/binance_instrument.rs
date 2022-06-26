use exc::service::instrument::FetchInstrumentsService;
use exc_binance::Binance;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_instrument=debug,exc_binance=debug".into()),
        ))
        .init();

    let mut binance = Binance::usd_margin_futures().connect().into_exchange();
    let mut stream = binance.fetch_instruments("").await?;
    while let Some(meta) = stream.next().await {
        let meta = meta?;
        tracing::info!("{meta}");
    }
    Ok(())
}
