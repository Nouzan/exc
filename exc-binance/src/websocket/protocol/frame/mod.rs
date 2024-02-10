#![allow(clippy::large_enum_variant)]

use std::fmt;

use futures::{future, stream, Sink, SinkExt, Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::websocket::error::WsError;

use self::{account::AccountEvent, agg_trade::AggTrade, book_ticker::BookTicker};

/// Aggregate trade.
pub mod agg_trade;

/// Book ticker.
pub mod book_ticker;

/// Account.
pub mod account;

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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Name {
    inst: Option<String>,
    channel: String,
}

impl Name {
    pub(crate) fn new(channel: &str) -> Self {
        Self {
            inst: None,
            channel: channel.to_string(),
        }
    }

    pub(crate) fn inst(mut self, inst: &str) -> Self {
        self.inst = Some(inst.to_string());
        self
    }

    /// Aggrated trade
    pub fn agg_trade(inst: &str) -> Self {
        Self {
            inst: Some(inst.to_string()),
            channel: "aggTrade".to_string(),
        }
    }

    /// Trade
    pub fn trade(inst: &str) -> Self {
        Self {
            inst: Some(inst.to_string()),
            channel: "trade".to_string(),
        }
    }

    /// Book ticker
    pub fn book_ticker(inst: &str) -> Self {
        Self {
            inst: Some(inst.to_string()),
            channel: "bookTicker".to_string(),
        }
    }

    /// Depth
    pub fn depth(inst: &str, levels: &str, rate: &str) -> Self {
        Self {
            inst: Some(inst.to_string()),
            channel: format!("depth{levels}@{rate}"),
        }
    }

    /// Listen key expired.
    pub fn listen_key_expired() -> Self {
        Self::new("listenKeyExpired")
    }

    /// Order trade update.
    pub fn order_trade_update(inst: &str) -> Self {
        Self::new("orderTradeUpdate").inst(inst)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(inst) = self.inst.as_ref() {
            write!(f, "{}@{}", inst, self.channel)
        } else {
            write!(f, "{}", self.channel)
        }
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
    pub(super) params: Vec<Name>,
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

    /// Unssubscribe a stream.
    pub fn unsubscribe(id: usize, stream: Name) -> Self {
        Self {
            id,
            method: Op::Unsubscribe,
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

impl ResponseFrame {
    pub(super) fn is_close_stream(&self) -> bool {
        false
    }
}

/// Server frame.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ServerFrame {
    /// Response.
    Response(ResponseFrame),
    /// Stream.
    Stream(StreamFrame),
    /// Empty.
    Empty,
}

impl ServerFrame {
    fn health(self) -> Result<Self, WsError> {
        match &self {
            Self::Stream(f) => match &f.data {
                StreamFrameKind::AccountEvent(AccountEvent::ListenKeyExpired { ts }) => {
                    Err(WsError::ListenKeyExpired(*ts))
                }
                _ => Ok(self),
            },
            _ => Ok(self),
        }
    }
}

/// Payload that with stream name.
pub trait Nameable {
    /// Get name.
    fn to_name(&self) -> Name;
}

/// Stream frame kind.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum StreamFrameKind {
    /// Aggregate trade.
    AggTrade(AggTrade),
    /// Book ticker.
    BookTicker(BookTicker),
    /// Account event.
    AccountEvent(AccountEvent),
    /// Unknwon.
    Unknwon(serde_json::Value),
}

/// Stream frame.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamFrame {
    /// Stream name.
    pub stream: String,
    /// Kind.
    pub data: StreamFrameKind,
}

impl StreamFrame {
    /// Get stream name.
    pub fn to_name(&self) -> Option<Name> {
        match &self.data {
            StreamFrameKind::AggTrade(f) => Some(f.to_name()),
            StreamFrameKind::BookTicker(f) => Some(f.to_name()),
            StreamFrameKind::AccountEvent(e) => Some(e.to_name()),
            StreamFrameKind::Unknwon(_) => None,
        }
    }
}

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
            let f = serde_json::from_str::<ServerFrame>(&msg)
                .map_err(WsError::from)
                .and_then(ServerFrame::health);
            future::ready(f)
        })
}

#[cfg(test)]
mod test {
    use futures::{pin_mut, TryStreamExt};
    use tower::ServiceExt;

    use crate::{types::Name, Binance, Request};

    use super::agg_trade::AggTrade;
    use super::book_ticker::BookTicker;

    #[tokio::test]
    async fn test_aggregate_trade() -> anyhow::Result<()> {
        let mut api = Binance::usd_margin_futures().connect();
        let stream = (&mut api)
            .oneshot(Request::subscribe(Name::agg_trade("btcusdt")))
            .await?
            .into_stream::<AggTrade>()?;
        pin_mut!(stream);
        let trade = stream.try_next().await?.unwrap();
        println!("{trade:?}");
        Ok(())
    }

    #[tokio::test]
    async fn test_book_ticker() -> anyhow::Result<()> {
        let mut api = Binance::usd_margin_futures().connect();
        let stream = (&mut api)
            .oneshot(Request::subscribe(Name::book_ticker("btcusdt")))
            .await?
            .into_stream::<BookTicker>()?;
        pin_mut!(stream);
        let trade = stream.try_next().await?.unwrap();
        println!("{trade:?}");
        Ok(())
    }
}
