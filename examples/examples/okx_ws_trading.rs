use exc::{
    okx::{key::OkxKey, Okx},
    types::{Place, PlaceOrderOptions},
    IntoExc, {CheckOrderService, TradingService},
};
use rust_decimal_macros::dec;
use std::{env::var, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,okx_ws_trading=debug,exc_okx=trace".into()),
        ))
        .init();

    let key = OkxKey {
        apikey: var("OKX_APIKEY")?,
        secretkey: var("OKX_SECRETKEY")?,
        passphrase: var("OKX_PASSPHRASE")?,
    };

    let mut okx = Okx::endpoint()
        .private(key)
        .ws_request_timeout(Duration::from_secs(5))
        .connect()
        .into_exc();

    let inst = "DOGE-USDT";
    let id = okx
        .place_with_opts(
            &Place::with_size(dec!(10)).limit(dec!(0.01)),
            &PlaceOrderOptions::new(inst),
        )
        .await?
        .id;
    tracing::info!("id={id:?}");
    let order = okx.check(inst, &id).await?;
    tracing::info!("order={order:?}");
    okx.cancel(inst, &id).await?;
    tracing::info!("cancelled");
    let order = okx.check(inst, &id).await?;
    tracing::info!("order={order:?}");
    Ok(())
}
