use std::{pin::Pin, time::Duration};

use super::error::WsError;
use super::request::WsRequest;
use super::response::WsResponse;
use exc::transport::websocket::WsStream;
use futures::{Sink, Stream};
use tokio_tower::multiplex::Client as Multiplex;

trait Transport: Sink<WsRequest, Error = WsError> + Stream<Item = Result<WsResponse, WsError>> {}

impl<T> Transport for T
where
    T: Sink<WsRequest, Error = WsError>,
    T: Stream<Item = Result<WsResponse, WsError>>,
{
}

type BoxTransport = Pin<Box<dyn Transport + Send>>;

struct Protocol {
    transport: BoxTransport,
}

/// Binance websocket api service.
pub struct BinanceWsApi {
    svc: Multiplex<Protocol, WsError, WsRequest>,
}

impl BinanceWsApi {
    pub(crate) async fn init(websocket: WsStream, timeout: Duration) -> Result<Self, WsError> {
        let svc = Multiplex::with_error_handler(transport, |err| {
            tracing::error!("protocol error: {err}");
        });
        Ok(Self { svc })
    }
}
