use std::{
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::{clap_derive::ArgEnum, Parser};
use exc::{
    types::{OrderId, Place},
    CheckOrderService, SubscribeOrdersService, TradingService,
};
use exc_binance::{Binance, SpotOptions};
use futures::StreamExt;
use humantime::{format_duration, FormattedDuration};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Clone, Copy, ArgEnum)]
enum MarginOp {
    Loan,
    Repay,
}

impl From<MarginOp> for exc_binance::MarginOp {
    fn from(op: MarginOp) -> Self {
        match op {
            MarginOp::Loan => Self::Loan,
            MarginOp::Repay => Self::Repay,
        }
    }
}

#[derive(Parser)]
struct Args {
    #[clap(long, env)]
    exchange: String,
    #[clap(long, env)]
    key: String,
    inst: String,
    #[clap(long, short)]
    exec: Option<Vec<String>>,
    #[clap(long, short)]
    script: Option<PathBuf>,
    #[clap(long, arg_enum)]
    buy_margin: Option<MarginOp>,
    #[clap(long, arg_enum)]
    sell_margin: Option<MarginOp>,
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
        millis: u64,
    },
}

#[derive(Debug, Deserialize)]
struct Script {
    exec: Vec<Op>,
}

fn rtt(begin: Instant) -> FormattedDuration {
    format_duration(Instant::now().duration_since(begin))
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

    let mut exc = match args.exchange.as_str() {
        "binance-u" => {
            let key = serde_json::from_str(&args.key)?;
            Binance::usd_margin_futures().private(key).connect_exc()
        }
        "binance-s" => {
            let key = serde_json::from_str(&args.key)?;
            let options = match (args.buy_margin, args.sell_margin) {
                (None, None) => SpotOptions::default(),
                (buy, sell) => {
                    SpotOptions::with_margin(buy.map(|o| o.into()), sell.map(|o| o.into()))
                }
            };
            Binance::spot_with_options(options)
                .private(key)
                .connect_exc()
        }
        exchange => {
            anyhow::bail!("unsupported exchange: {exchange}");
        }
    };

    let mut orders_provider = exc.clone();
    let shared_inst = inst.clone();
    tokio::spawn(async move {
        let mut revision = 0;
        loop {
            revision += 1;
            match orders_provider.subscribe_orders(&shared_inst).await {
                Ok(mut orders) => {
                    while let Some(t) = orders.next().await {
                        match t {
                            Ok(t) => {
                                tracing::info!("[*] watch: {t:#?}");
                            }
                            Err(err) => {
                                tracing::error!("stream error: {err}[{revision}]");
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("request error: {err}[{revision}]");
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    for (idx, op) in execs.into_iter().enumerate() {
        let begin = Instant::now();
        match op {
            Op::Wait { millis } => {
                tracing::info!("[{idx}] wait for {millis}ms");
                tokio::time::sleep(Duration::from_millis(millis)).await;
            }
            Op::Check { name } => match exc.check(&inst, &OrderId::from(name)).await {
                Ok(update) => {
                    tracing::info!("[{idx}] check(rtt={}): {update:#?}", rtt(begin))
                }
                Err(err) => tracing::error!("[{idx}] check(rtt={}): {err}", rtt(begin)),
            },
            Op::Cancel { name } => {
                match exc.cancel(&inst, &OrderId::from(name)).await {
                    Ok(cancelled) => {
                        tracing::info!("[{idx}] cancel(rtt={}): {cancelled:#?}", rtt(begin))
                    }
                    Err(err) => tracing::error!("[{idx}] cancel(rtt={}): {err}", rtt(begin)),
                };
            }
            Op::Market { name, size } => {
                match exc.place(&inst, &Place::with_size(size), Some(&name)).await {
                    Ok(placed) => tracing::info!("[{idx}] market(rtt={}): {placed:#?}", rtt(begin)),
                    Err(err) => tracing::error!("[{idx}] market(rtt={}): {err}", rtt(begin)),
                }
            }
            Op::Limit { name, price, size } => {
                match exc
                    .place(&inst, &Place::with_size(size).limit(price), Some(&name))
                    .await
                {
                    Ok(placed) => tracing::info!("[{idx}] limit(rtt={}): {placed:#?}", rtt(begin)),
                    Err(err) => tracing::error!("[{idx}] limit(rtt={}): {err}", rtt(begin)),
                }
            }
            Op::PostOnly { name, price, size } => {
                match exc
                    .place(&inst, &Place::with_size(size).post_only(price), Some(&name))
                    .await
                {
                    Ok(placed) => {
                        tracing::info!("[{idx}] post-only(rtt={}): {placed:#?}", rtt(begin))
                    }
                    Err(err) => tracing::error!("[{idx}] post-only(rtt={}): {err}", rtt(begin)),
                };
            }
        }
    }
    Ok(())
}
