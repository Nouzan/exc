use std::fmt;

use futures::{future, stream, Sink, SinkExt, Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::websocket::error::WsError;

/// Operations.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Op {
    /// Subscribe.
    Subscribe,
    /// Unsubscribe.
    Unsubscribe,
}

/// Stream name.
#[derive(Debug, Clone)]
pub struct Name {
    inst: String,
    channel: String,
}

impl Name {
    /// aggrated trade
    pub fn agg_trade(inst: &str) -> Self {
        Self {
            inst: inst.to_string(),
            channel: "aggTrade".to_string(),
        }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.inst, self.channel)
    }
}

/// Request frame.
#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct RequestFrame {
    /// Id.
    pub id: usize,
    /// Method.
    pub method: Op,
    /// Params.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub params: Vec<Name>,
}

impl RequestFrame {
    /// Subscribe to a stream.
    pub fn subscribe(id: usize, stream: Name) -> Self {
        Self {
            id,
            method: Op::Subscribe,
            params: vec![stream],
        }
    }
}

/// Response frame.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseFrame {
    /// Id.
    pub id: usize,
    /// Result.
    #[serde(default)]
    pub result: Option<serde_json::Value>,
}

/// Server frame.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ServerFrame {
    /// Response.
    Response(ResponseFrame),
    /// Stream.
    Stream(serde_json::Value),
}

/// Stream frame.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamFrame {}

/// Frame protocol layer.
pub fn layer<T>(
    transport: T,
) -> impl Sink<RequestFrame, Error = WsError> + Stream<Item = Result<ServerFrame, WsError>>
where
    T: Sink<String, Error = WsError>,
    T: Stream<Item = Result<String, WsError>>,
{
    transport
        .with_flat_map(|f| {
            let msg = serde_json::to_string(&f).map_err(WsError::from);
            stream::once(future::ready(msg))
        })
        .and_then(|msg| {
            let f = serde_json::from_str::<ServerFrame>(&msg).map_err(WsError::from);
            future::ready(f)
        })
}
