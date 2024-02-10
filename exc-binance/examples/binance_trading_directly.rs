use std::time::Duration;

use exc_binance::{
    http::{
        request::trading::{usd_margin_futures::PlaceOrder, CancelOrder, GetOrder, GetOrderInner},
        response::trading::Order,
    },
    types::trading::{OrderSide, OrderType, PositionSide, TimeInForce},
    Binance, Request,
};
use rust_decimal_macros::dec;
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,binance_trading=debug,exc_binance=trace".into()),
        ))
        .init();

    let key = std::env::var("BINANCE_KEY")?;
    let key = serde_json::from_str(&key)?;

    let mut api = Binance::usd_margin_futures()
        .ws_keep_alive_timeout(Duration::from_secs(30))
        .private(key)
        .ws_listen_key_retry(5)
        .ws_listen_key_refresh_interval(Duration::from_secs(60))
        .connect();
    api.ready().await?;
    let res = api
        .call(Request::with_rest_payload(PlaceOrder {
            symbol: "btcusdt".to_string(),
            side: OrderSide::Sell,
            position_side: PositionSide::Both,
            order_type: OrderType::Limit,
            reduce_only: None,
            quantity: Some(dec!(0.001)),
            price: Some(dec!(30_000)),
            new_client_order_id: None,
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            time_in_force: Some(TimeInForce::Gtc),
            working_type: None,
            price_protect: None,
            new_order_resp_type: None,
        }))
        .await?
        .into_response::<Order>()?;
    tracing::info!("{res:#?}");
    let id = res.id();
    let symbol = res.symbol();
    api.ready().await?;
    let res = api
        .call(Request::with_rest_payload(GetOrder {
            inner: GetOrderInner {
                symbol: symbol.to_string(),
                order_id: Some(id),
                orig_client_order_id: None,
                client_order_id: None,
            },
        }))
        .await?
        .into_response::<Order>()?;
    tracing::info!("{res:#?}");
    api.ready().await?;
    let res = api
        .call(Request::with_rest_payload(CancelOrder {
            inner: GetOrderInner {
                symbol: symbol.to_string(),
                order_id: Some(id),
                orig_client_order_id: None,
                client_order_id: None,
            },
        }))
        .await?
        .into_response::<Order>()?;
    tracing::info!("{res:#?}");
    Ok(())
}
