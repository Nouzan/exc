use clap::Parser;
use exc_binance::{
    http::{
        request::{GetSubAccountAssets, ListSubAccounts},
        response::{SubAccountBalances, SubAccounts},
    },
    Binance, Request,
};
use tower::{Service, ServiceExt};

#[derive(Parser)]
struct Args {
    #[clap(long, env)]
    binance_main: String,
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
    let key = serde_json::from_str(&args.binance_main)?;

    let mut api = Binance::spot().private(key).connect();
    api.ready().await?;
    let sub_accounts = api
        .call(Request::with_rest_payload(ListSubAccounts {
            ..Default::default()
        }))
        .await?
        .into_response::<SubAccounts>()?
        .sub_accounts;
    for account in sub_accounts {
        println!("{}", account.email);
        api.ready().await?;
        let assets = api
            .call(Request::with_rest_payload(GetSubAccountAssets {
                email: account.email,
            }))
            .await?
            .into_response::<SubAccountBalances>()?
            .balances;
        for asset in assets {
            if !asset.free.is_zero() || !asset.locked.is_zero() {
                println!("{}: {},{}", asset.asset, asset.free, asset.locked);
            }
        }
    }
    Ok(())
}
