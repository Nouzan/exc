use clap::Parser;
use exc::{
    types::SubscribeOrders, ExcService, IntoExc, SubscribeOrdersService, SubscribeTickersService,
    TradeBidAskService,
};
use exc_binance::Binance;
use futures::StreamExt;
use std::time::Duration;
use tracing::{instrument, Instrument};
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

#[derive(Parser)]
struct Args {
    inst: String,
    #[clap(long, env)]
    binance_key: String,
    #[clap(long, short, default_value = "12h")]
    reconnect: humantime::Duration,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::new(
        std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "error,binance_orders=debug,exc_binance=debug".into()),
    );
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_sampler(opentelemetry::sdk::trace::Sampler::AlwaysOn)
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "binance-orders"),
                ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_line_number(true);

    Registry::default()
        .with(env_filter)
        .with(otel_layer)
        .with(fmt_layer)
        .try_init()?;

    tokio::select! {
        res = start() => {
            res?;
        },
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("ctrl + c");
        }
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

#[instrument(skip(market))]
async fn stream_worker(mut market: impl SubscribeTickersService, inst: &str) -> anyhow::Result<()> {
    let mut revision = 0;
    loop {
        revision += 1;
        match market
            .subscribe_tickers(inst)
            .instrument(tracing::info_span!("subscribe", inst))
            .await
        {
            Ok(mut stream) => {
                while let Some(ticker) = stream
                    .next()
                    .instrument(tracing::info_span!("fetch_next", inst))
                    .await
                {
                    let span = tracing::info_span!("hanlde_ticker", inst);
                    let _enter = span.enter();
                    match ticker {
                        Ok(ticker) => {
                            if !ticker.size.is_zero() {
                                tracing::info!(rev = revision, %inst, "{ticker}");
                            }
                        }
                        Err(err) => {
                            tracing::error!(rev = revision, %inst, "stream error: {err}");
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                tracing::error!(
                    rev = revision,
                    %inst,
                    "request new stream error: {err}"
                );
            }
        }
    }
}

#[instrument]
async fn start() -> anyhow::Result<()> {
    let args = Args::from_args();
    let key = serde_json::from_str(&args.binance_key)?;

    let endpoint = std::env::var("ENDPOINT").unwrap_or_else(|_| String::from("binance-u"));
    let mut endpoint = match endpoint.as_str() {
        "binance-u" => Binance::usd_margin_futures(),
        "binance-s" => Binance::spot(),
        _ => anyhow::bail!("unsupported"),
    };

    let binance = endpoint
        .private(key)
        .ws_listen_key_stop_refreshing_after(args.reconnect.into())
        .connect()
        .into_exc();

    let inst = args.inst.clone();
    let market = binance
        .clone()
        .into_subscribe_tickers()
        .into_retry(Duration::from_secs(30));

    let (_cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(
        async move {
            tokio::select! {
                _ = stream_worker(market, &inst) => {

                },
                _ = cancel_rx => {

                }
            }
        }
        .in_current_span(),
    );

    let mut revision = 0;
    let mut orders = ExcService::<SubscribeOrders>::into_retry(binance, Duration::from_secs(30));
    loop {
        revision += 1;
        match orders.subscribe_orders(&args.inst).await {
            Ok(mut orders) => {
                while let Some(t) = orders.next().await {
                    match t {
                        Ok(t) => {
                            tracing::info!(rev = revision, "{t:#?}");
                        }
                        Err(err) => {
                            tracing::error!(rev = revision, "stream error: {err}");
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                tracing::error!(rev = revision, "request error: {err}");
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
