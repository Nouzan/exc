use clap::Parser;
use exc::binance::Binance;
use exc::{types::Place, CheckOrderService, IntoExc, TradingService};
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

    let endpoint = std::env::var("ENDPOINT").unwrap_or_else(|_| String::from("binance-u"));
    let mut endpoint = match endpoint.as_str() {
        "binance-u" => Binance::usd_margin_futures(),
        "binance-s" => Binance::spot(),
        _ => anyhow::bail!("unsupported"),
    };

    let mut binance = endpoint.private(key).connect().into_exc();

    let place = Place::with_size(dec!(200)).post_only(dec!(0.05));
    let placed = binance.place(&args.inst, &place, Some("test")).await?;
    tracing::info!("placed={placed:?}");
    let id = placed.id;
    let order = binance.check(&args.inst, &id).await?;
    tracing::info!("checked={order:?}");
    let cancelled = binance.cancel(&args.inst, &id).await?;
    tracing::info!("cancelled={cancelled:?}");
    let order = binance.check(&args.inst, &id).await?;
    tracing::info!("checked={order:?}");
    Ok(())
}
