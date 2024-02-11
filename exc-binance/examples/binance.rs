use std::time::Duration;

use exc_binance::{
    http::request::{utils::Ping, Payload, RestRequest},
    websocket::{
        protocol::frame::{agg_trade::AggTrade, Name},
        request::WsRequest,
    },
    Binance, Request,
};
use futures::StreamExt;
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "error,binance=debug".into()),
        ))
        .init();
    let mut api = Binance::usd_margin_futures()
        .ws_keep_alive_timeout(Duration::from_secs(30))
        .connect();
    api.ready().await?;
    let mut stream = api
        .call(Request::Ws(WsRequest::subscribe_stream(Name::agg_trade(
            "btcusdt",
        ))))
        .await?
        .into_stream::<AggTrade>()
        .unwrap()
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
    let resp = api
        .call(Request::Http(RestRequest::from(Payload::new(Ping))))
        .await?;
    tracing::info!("ping response: {resp:?}");
    tokio::time::sleep(Duration::from_secs(2)).await;
    let mut count = 1;
    loop {
        count += 1;
        api.ready().await?;
        match api
            .call(Request::Ws(WsRequest::subscribe_stream(Name::agg_trade(
                "btcusdt",
            ))))
            .await
        {
            Ok(resp) => {
                let mut stream = resp.into_stream::<AggTrade>().unwrap().boxed();
                while let Some(trade) = stream.next().await {
                    match trade {
                        Ok(trade) => {
                            counter += 1;
                            tracing::info!("[{count}]trade={trade:?}");
                        }
                        Err(err) => {
                            tracing::error!("[{count}]error={err}");
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                tracing::error!("[{count}] request error: {err}");
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
