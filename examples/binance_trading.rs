use clap::Parser;
use exc::{types::Place, CheckOrderService, IntoExc, TradingService};
use exc_binance::Binance;
use rust_decimal_macros::dec;

#[derive(Parser)]
struct Args {
    #[clap(long, env)]
    binance_key: String,
    inst: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_trading=debug,exc_binance=debug".into()),
        ))
        .init();

    let args = Args::from_args();
    let key = serde_json::from_str(&args.binance_key)?;

    let mut binance = Binance::usd_margin_futures()
        .private(key)
        .connect()
        .into_exc();

    let place = Place::with_size(dec!(-0.001)).post_only(dec!(30_000));
    let id = binance.place(&args.inst, &place, Some("test")).await?;
    tracing::info!("placed={}", id.as_str());
    let order = binance.check(&args.inst, &id).await?;
    tracing::info!("checked={order:?}");
    binance.cancel(&args.inst, &id).await?;
    let order = binance.check(&args.inst, &id).await?;
    tracing::info!("cancelled={order:?}");
    Ok(())
}
