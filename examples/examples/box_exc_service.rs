use std::str::FromStr;

use clap::Parser;
use exc::{core::types::Ticker, ExcExt, SubscribeTickersService};
use futures::{Stream, TryStreamExt};

struct Exchange {
    ticker: Box<dyn SubscribeTickersService>,
}

impl Exchange {
    fn binance() -> Self {
        let binance = exc::Binance::spot().connect_exc().into_subscribe_tickers();
        Self {
            ticker: Box::new(binance),
        }
    }

    fn okx() -> Self {
        let okx = exc::Okx::endpoint().connect_exc();
        Self {
            ticker: Box::new(okx),
        }
    }

    async fn subscribe_tickers(
        &mut self,
        inst: impl AsRef<str>,
    ) -> anyhow::Result<impl Stream<Item = exc::Result<Ticker>>> {
        let stream = self.ticker.subscribe_tickers(inst.as_ref()).await?;
        Ok(stream)
    }
}

impl FromStr for Exchange {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binance" => Ok(Self::binance()),
            "okx" => Ok(Self::okx()),
            s => anyhow::bail!("unsupported exchange: {s}"),
        }
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    exchange: String,
    #[arg(long)]
    inst: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;
    let mut exchange: Exchange = cli.exchange.parse()?;
    let mut stream = exchange.subscribe_tickers(&cli.inst).await?;
    while let Some(ticker) = stream.try_next().await? {
        println!("{ticker}");
    }
    Ok(())
}
