use std::{ops::RangeBounds, str::FromStr, time::Duration};

use clap::Parser;
use exc::{
    core::types::{Candle, SubscribeTickers, SubscribeTrades, Ticker, Trade},
    prelude::*,
    types::instrument::InstrumentMeta,
    util::instrument::PollInstrumentsLayer,
    ExcLayer, IntoExc,
};
use futures::{Stream, StreamExt, TryStreamExt};
use rust_decimal::Decimal;
use time::{macros::datetime, OffsetDateTime};
use tower::{
    layer::{layer_fn, util::Stack},
    ServiceExt,
};

struct Exchange {
    ticker: Box<dyn SubscribeTickersService>,
    trade: Box<dyn SubscribeTradesService>,
    inst: Box<dyn SubscribeInstrumentsService>,
    candle: Box<dyn FetchCandlesService>,
    reconnect: Box<dyn ReconnectService>,
}

impl Exchange {
    fn binance() -> Self {
        let binance = exc::Binance::usd_margin_futures().connect_exc();
        Self {
            ticker: Box::new(binance.clone().into_subscribe_tickers()),
            // We cannot clone `Binance` here because the user may subscribe to both ticker and trade
            // channels which will undelryingly subscribe trade channel twice.
            trade: Box::new(exc::Binance::usd_margin_futures().connect_exc()),
            inst: Box::new(binance.clone().layer(&Stack::new(
                ExcLayer::default(),
                PollInstrumentsLayer::new(Duration::from_secs(3600)),
            ))),
            candle: Box::new(binance.clone().into_fetch_candles_forward(1000)),
            reconnect: Box::new(binance),
        }
    }

    fn okx() -> Self {
        let okx = exc::Okx::endpoint().connect_exc();
        // We cannot clone `Okx` here because the user may subscribe to both ticker and trade
        // channels which will undelryingly subscribe to ticker channel twice.
        let trade_svc = exc::Okx::endpoint()
            .connect_exc()
            .layer(&layer_fn(|svc: exc::Okx| {
                svc.into_exc()
                    // We use `adapt` method to convert the request type to `SubscribeTickers`.
                    .adapt::<SubscribeTickers>()
                    .map_request(|req: SubscribeTrades| SubscribeTickers {
                        instrument: req.instrument,
                    })
                    .map_response(|res| {
                        res.map_ok(|t| Trade {
                            ts: t.ts,
                            price: t.last,
                            size: t.size,
                            // Just as an example, it is not recommended.
                            buy: t.buy.unwrap_or(true),
                        })
                        .boxed()
                    })
            }));
        Self {
            ticker: Box::new(okx.clone()),
            trade: Box::new(trade_svc),
            inst: Box::new(okx.clone().layer(&Stack::new(
                ExcLayer::default(),
                PollInstrumentsLayer::new(Duration::from_secs(3600)),
            ))),
            candle: Box::new(okx.clone().into_fetch_candles_backward(100)),
            reconnect: Box::new(okx),
        }
    }

    async fn subscribe_tickers(
        &mut self,
        inst: impl AsRef<str>,
    ) -> anyhow::Result<impl Stream<Item = exc::Result<Ticker>>> {
        Ok(self.ticker.subscribe_tickers(inst.as_ref()).await?)
    }

    async fn subscribe_trades(
        &mut self,
        inst: impl AsRef<str>,
    ) -> anyhow::Result<impl Stream<Item = exc::Result<Trade>>> {
        Ok(self.trade.subscribe_trades(inst.as_ref()).await?)
    }

    async fn subscribe_insts(
        &mut self,
        tag: impl AsRef<str>,
    ) -> anyhow::Result<impl Stream<Item = exc::Result<InstrumentMeta<Decimal>>>> {
        Ok(self.inst.subscribe_instruments(tag.as_ref()).await?)
    }

    async fn fetch_candles(
        &mut self,
        inst: impl AsRef<str>,
        period: Period,
        range: impl RangeBounds<OffsetDateTime>,
    ) -> anyhow::Result<impl Stream<Item = exc::Result<Candle>>> {
        Ok(self.candle.fetch_candles_range(inst, period, range).await?)
    }

    async fn reconnect(&mut self) -> anyhow::Result<()> {
        self.reconnect.reconnect().await?;
        Ok(())
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
    #[arg(long, default_value = "")]
    tag: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .init();
    let cli = Cli::try_parse()?;
    let mut exchange: Exchange = cli.exchange.parse()?;
    // Fetch candeles.
    let mut candles = exchange
        .fetch_candles(
            &cli.inst,
            Period::secs(60),
            datetime!(2023-05-14 00:00:00 +0)..datetime!(2023-05-14 00:02:00 +0),
        )
        .await?;
    while let Some(candle) = candles.try_next().await? {
        println!("candle: {candle}");
    }

    // Subscribe to tickers.
    let mut stream = exchange.subscribe_tickers(cli.inst.clone()).await?;
    let ticker_handle = tokio::spawn(async move {
        while let Some(ticker) = stream.try_next().await? {
            println!("ticker: {ticker}");
        }
        anyhow::Result::<_>::Ok(())
    });

    // Subscribe to trades.
    let mut stream = exchange.subscribe_trades(cli.inst.clone()).await?;
    let trade_handle = tokio::spawn(async move {
        while let Some(trade) = stream.try_next().await? {
            println!("trade: {trade}");
        }
        anyhow::Result::<_>::Ok(())
    });

    // Subscribe to instruments.
    let mut stream = exchange.subscribe_insts(cli.tag.clone()).await?;
    let inst_handle = tokio::spawn(async move {
        while let Some(inst) = stream.try_next().await? {
            println!("inst: {inst}");
        }
        anyhow::Result::<_>::Ok(())
    });
    // Just for an example.
    exchange.reconnect().await?;
    ticker_handle.await??;
    trade_handle.await??;
    inst_handle.await??;
    Ok(())
}
