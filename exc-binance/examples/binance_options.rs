use std::time::Duration;

use exc_binance::{
    websocket::{protocol::frame::Name, request::WsRequest},
    Binance, Request,
};
use futures::StreamExt;
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "error,binance_options=debug".into()),
        ))
        .init();
    let channel = std::env::var("CHANNEL").unwrap_or("index".to_string());
    let inst = std::env::var("INST")?;
    let mut api = Binance::european_options()
        .ws_keep_alive_timeout(Duration::from_secs(30))
        .connect();
    api.ready().await?;
    let mut stream = api
        .call(Request::Ws(WsRequest::subscribe_stream(
            Name::new(&channel).with_inst(&inst),
        )))
        .await?
        .into_stream::<serde_json::Value>()
        .unwrap()
        .boxed();
    while let Some(data) = stream.next().await {
        match data {
            Ok(data) => {
                tracing::info!("data={data:#?}");
            }
            Err(err) => {
                tracing::error!("error={err}");
                break;
            }
        }
    }
    Ok(())
}
