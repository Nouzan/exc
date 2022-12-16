use exc_okx::{
    http::types::request::{instruments::Instruments, Get, HttpRequest},
    Okx, OkxRequest,
};
use tower::{Service, ServiceExt};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Registry::default()
        .with(fmt::layer().with_line_number(true))
        .with(
            EnvFilter::builder()
                .with_default_directive("info".parse()?)
                .from_env_lossy(),
        )
        .init();
    let mut okx = Okx::endpoint().connect();
    okx.ready().await?;
    let resp = okx
        .call(OkxRequest::Http(HttpRequest::Get(Get::Instruments(
            Instruments::spot(),
        ))))
        .await?
        .http()?;

    for data in resp.data {
        println!("{data:?}");
    }
    Ok(())
}
