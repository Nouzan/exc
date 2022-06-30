use std::time::Duration;

use exc_binance::{
    websocket::protocol::frame::{account::AccountEvent, Name},
    Binance, Request,
};
use futures::{pin_mut, StreamExt};
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_private=debug,exc_binance=trace".into()),
        ))
        .init();

    let key = std::env::var("BINANCE_KEY")?;
    let key = serde_json::from_str(&key)?;

    let mut api = Binance::usd_margin_futures()
        .ws_keep_alive_timeout(Duration::from_secs(30))
        .private(key)
        .connect();
    // let mut count = 1;
    // loop {
    //     count += 1;
    //     api.ready().await?;
    //     match api
    //         .call(Request::Ws(WsRequest::subscribe(Name::agg_trade(
    //             "btcbusd",
    //         ))))
    //         .await
    //     {
    //         Ok(resp) => {
    //             let mut stream = resp.into_stream::<AggTrade>().unwrap().boxed();
    //             while let Some(trade) = stream.next().await {
    //                 match trade {
    //                     Ok(trade) => {
    //                         count += 1;
    //                         tracing::info!("[{count}]trade={trade:?}");
    //                     }
    //                     Err(err) => {
    //                         tracing::error!("[{count}]error={err}");
    //                         break;
    //                     }
    //                 }
    //             }
    //         }
    //         Err(err) => {
    //             tracing::error!("[{count}] request error: {err}");
    //         }
    //     }
    //     tokio::time::sleep(Duration::from_secs(1)).await;
    // }
    api.ready().await?;
    let stream = api
        .call(Request::subcribe_main(Name::listen_key_expired()))
        .await?
        .into_stream::<AccountEvent>()?;
    pin_mut!(stream);
    while let Some(e) = stream.next().await {
        let e = e?;
        tracing::info!("{e:?}");
    }
    Ok(())
}
