use std::time::Duration;

use exc_binance::websocket::{
    error::WsError,
    protocol::{
        frame::{self, Name, RequestFrame},
        keep_alive, BinanceWsApi,
    },
};
use futures::{SinkExt, StreamExt, TryStreamExt};
use http::Uri;
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,exc_binance=trace,binance_ws=debug".into()),
        ))
        .init();
    let ws = exc::transport::websocket::connector::WsConnector::default()
        .oneshot(Uri::from_static(
            "wss://fstream.binance.com/ws/bnbusdt@aggTrade",
        ))
        .await?
        .sink_map_err(WsError::from)
        .map_err(WsError::from);
    let transport = keep_alive::layer(ws, Duration::from_secs(30));
    let transport = frame::layer(transport);
    let (mut tx, mut rx) = transport.split();
    tokio::spawn(async move {
        while let Some(msg) = rx.next().await {
            match msg {
                Ok(msg) => {
                    tracing::info!("msg={msg:?}");
                }
                Err(err) => {
                    tracing::error!("error={err}");
                    break;
                }
            }
        }
    });
    tx.send(RequestFrame::subscribe(2, Name::agg_trade("btcusdt")))
        .await?;
    tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    Ok(())
}
