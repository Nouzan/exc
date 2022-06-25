use std::time::Duration;

use exc_binance::websocket::{
    protocol::{
        frame::{agg_trade::AggTrade, Name},
        BinanceWsApi,
    },
    request::WsRequest,
};
use futures::StreamExt;
use http::Uri;
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,exc_binance=trace,binance_ws_protocol=debug".into()),
        ))
        .init();
    let ws = exc::transport::websocket::connector::WsConnector::default()
        .oneshot(Uri::from_static(
            "wss://fstream.binance.com/ws/bnbusdt@aggTrade",
        ))
        .await?;
    let mut api = BinanceWsApi::with_websocket(ws, Duration::from_secs(30))?;
    api.ready().await?;
    let mut stream = api
        .call(WsRequest::subscribe(Name::agg_trade("btcusdt")))
        .await?
        .into_stream::<AggTrade>()
        .await?
        .boxed();
    let mut counter = 0;
    while let Some(trade) = stream.next().await {
        match trade {
            Ok(trade) => {
                counter += 1;
                tracing::info!("[1]trade={trade:?}");
                if counter > 100 {
                    break;
                }
            }
            Err(err) => {
                tracing::error!("[1]error={err}");
                break;
            }
        }
    }
    drop(stream);
    api.ready().await?;
    let mut stream = api
        .call(WsRequest::subscribe(Name::agg_trade("btcusdt")))
        .await?
        .into_stream::<AggTrade>()
        .await?
        .boxed();
    while let Some(trade) = stream.next().await {
        match trade {
            Ok(trade) => {
                counter += 1;
                tracing::info!("[2]trade={trade:?}");
            }
            Err(err) => {
                tracing::error!("[2]error={err}");
                break;
            }
        }
    }
    Ok(())
}
