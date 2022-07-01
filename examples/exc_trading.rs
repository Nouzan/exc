use std::{io::Read, path::PathBuf, time::Duration};

use clap::Parser;
use exc::{
    types::{OrderId, Place},
    CheckOrderService, IntoExc, TradingService,
};
use exc_binance::Binance;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Parser)]
struct Args {
    #[clap(long, env)]
    binance_key: String,
    inst: String,
    #[clap(long, short)]
    exec: Option<Vec<String>>,
    #[clap(long, short)]
    script: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Op {
    Market {
        name: String,
        size: Decimal,
    },
    Limit {
        name: String,
        price: Decimal,
        size: Decimal,
    },
    PostOnly {
        name: String,
        price: Decimal,
        size: Decimal,
    },
    Cancel {
        name: String,
    },
    Check {
        name: String,
    },
    Wait {
        seconds: u64,
    },
}

#[derive(Debug, Deserialize)]
struct Script {
    exec: Vec<Op>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "error,exc_trading=debug".into()),
        ))
        .init();

    let args = Args::from_args();
    let mut execs = Vec::default();
    if let Some(ops) = args.exec {
        for op in ops {
            let exec = toml::from_str(&op)?;
            execs.push(exec);
        }
    } else if let Some(path) = args.script {
        let f = std::fs::OpenOptions::new().read(true).open(path)?;
        let mut r = std::io::BufReader::new(f);
        let mut buf = Vec::default();
        r.read_to_end(&mut buf)?;
        execs = toml::from_slice::<Script>(&buf)?.exec;
    } else {
        anyhow::bail!("must provide one of `--exec` and `--script`");
    }

    let inst = args.inst;
    let key = serde_json::from_str(&args.binance_key)?;

    let mut exc = Binance::usd_margin_futures()
        .private(key)
        .connect()
        .into_exc();

    // let mut orders_provider = exc.clone();
    // let shared_inst = inst.clone();
    // tokio::spawn(async move {
    //     let mut revision = 0;
    //     loop {
    //         revision += 1;
    //         match orders_provider.subscribe_orders(&shared_inst).await {
    //             Ok(mut orders) => {
    //                 while let Some(t) = orders.next().await {
    //                     match t {
    //                         Ok(t) => {
    //                             tracing::info!("{t:?}[{revision}]");
    //                         }
    //                         Err(err) => {
    //                             tracing::error!("stream error: {err}[{revision}]");
    //                             break;
    //                         }
    //                     }
    //                 }
    //             }
    //             Err(err) => {
    //                 tracing::error!("request error: {err}[{revision}]");
    //             }
    //         }
    //         tokio::time::sleep(Duration::from_secs(1)).await;
    //     }
    // });

    for (idx, op) in execs.into_iter().enumerate() {
        match op {
            Op::Wait { seconds } => {
                println!("[{idx}] wait for {seconds}s");
                tokio::time::sleep(Duration::from_secs(seconds)).await;
            }
            Op::Check { name } => {
                let update = exc.check(&inst, &OrderId::from(name)).await?;
                println!("[{idx}] check: {update:#?}");
            }
            Op::Cancel { name } => {
                let cancelled = exc.cancel(&inst, &OrderId::from(name)).await?;
                println!("[{idx}] cancel: {cancelled:#?}");
            }
            Op::Market { name, size } => {
                let placed = exc
                    .place(&inst, &Place::with_size(size), Some(&name))
                    .await?;
                println!("[{idx}] market: {placed:#?}");
            }
            Op::Limit { name, price, size } => {
                let placed = exc
                    .place(&inst, &Place::with_size(size).limit(price), Some(&name))
                    .await?;
                println!("[{idx}] limit: {placed:#?}");
            }
            Op::PostOnly { name, price, size } => {
                let placed = exc
                    .place(&inst, &Place::with_size(size).post_only(price), Some(&name))
                    .await?;
                println!("[{idx}] post-only: {placed:#?}");
            }
        }
    }
    Ok(())
}
