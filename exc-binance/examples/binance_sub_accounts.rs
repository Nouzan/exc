use clap::Parser;
use exc_binance::{http::request::ListSubAccounts, Binance, Request};
use tower::{Service, ServiceExt};

#[derive(Parser)]
struct Args {
    #[clap(long, env)]
    binance_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_sub_accounts=debug,exc_binance=trace".into()),
        ))
        .init();

    let args = Args::from_args();
    let key = serde_json::from_str(&args.binance_key)?;

    let mut api = Binance::spot().private(key).connect();
    api.ready().await?;
    let res = api
        .call(Request::with_rest_payload(ListSubAccounts {
            ..Default::default()
        }))
        .await?;
    tracing::info!(?res, "got response");
    Ok(())
}
