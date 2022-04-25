use exc_okx::websocket::WsEndpoint;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "basic_okx=debug,exc_okx=debug".into()),
        ))
        .init();

    let mut channel = WsEndpoint::default().connect().await?;
    channel.ready().await?;
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    Ok(())
}
