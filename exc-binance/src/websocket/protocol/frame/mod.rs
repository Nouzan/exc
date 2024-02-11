#![allow(clippy::large_enum_variant)]

use std::fmt;

use exc_core::ExchangeError;
use futures::{future, stream, Sink, SinkExt, Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::websocket::error::WsError;

use self::{account::AccountEvent, agg_trade::AggTrade, book_ticker::BookTicker};

/// Aggregate trade.
pub mod agg_trade;

/// Trade.
pub mod trade;

/// Book ticker.
pub mod book_ticker;

/// Depth.
pub mod depth;

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
    /// Create a new stream name.
    pub fn new(channel: &str) -> Self {
        Self {
            inst: None,
            channel: channel.to_string(),
        }
    }

    /// Set instrument.
    pub fn with_inst(mut self, inst: &str) -> Self {
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
        Self::new("orderTradeUpdate").with_inst(inst)
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

    fn break_down(self) -> Vec<Self> {
        match &self {
            Self::Empty | Self::Response(_) => vec![self],
            Self::Stream(f) => match &f.data {
                StreamFrameKind::OptionsOrderUpdate(_) => {
                    let Self::Stream(f) = self else {
                        unreachable!()
                    };
                    let StreamFrameKind::OptionsOrderUpdate(update) = f.data else {
                        unreachable!()
                    };
                    let stream = f.stream;
                    update
                        .order
                        .into_iter()
                        .map(|o| {
                            let frame = StreamFrame {
                                stream: stream.clone(),
                                data: StreamFrameKind::OptionsOrder(o),
                            };
                            Self::Stream(frame)
                        })
                        .collect()
                }
                _ => vec![self],
            },
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
#[non_exhaustive]
pub enum StreamFrameKind {
    /// Aggregate trade.
    AggTrade(AggTrade),
    /// Trade.
    Trade(trade::Trade),
    /// Book ticker.
    BookTicker(BookTicker),
    /// Depth.
    Depth(depth::Depth),
    /// Account event.
    AccountEvent(AccountEvent),
    /// Options Order Update.
    OptionsOrder(account::OptionsOrder),
    /// Options Order Trade Update.
    OptionsOrderUpdate(account::OptionsOrderUpdate),
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
            StreamFrameKind::Trade(f) => Some(f.to_name()),
            StreamFrameKind::BookTicker(f) => Some(f.to_name()),
            StreamFrameKind::Depth(_) => {
                let (inst, channel) = self.stream.split_once('@')?;
                Some(Name {
                    inst: Some(inst.to_string()),
                    channel: channel.to_string(),
                })
            }
            StreamFrameKind::AccountEvent(e) => Some(e.to_name()),
            StreamFrameKind::OptionsOrder(e) => Some(e.to_name()),
            StreamFrameKind::OptionsOrderUpdate(_) => None,
            StreamFrameKind::Unknwon(_) => {
                let (inst, channel) = self.stream.split_once('@')?;
                Some(Name {
                    inst: Some(inst.to_string()),
                    channel: channel.to_string(),
                })
            }
        }
    }
}

impl TryFrom<StreamFrame> for serde_json::Value {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        match frame.data {
            StreamFrameKind::Unknwon(v) => Ok(v),
            _ => Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}"))),
        }
    }
}

/// Trade frame.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub enum TradeFrame {
    /// Aggregate trade.
    AggTrade(AggTrade),
    /// Trade.
    Trade(trade::Trade),
}

impl TryFrom<StreamFrame> for TradeFrame {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        match frame.data {
            StreamFrameKind::AggTrade(trade) => Ok(Self::AggTrade(trade)),
            StreamFrameKind::Trade(trade) => Ok(Self::Trade(trade)),
            _ => Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}"))),
        }
    }
}

impl TryFrom<TradeFrame> for exc_core::types::Trade {
    type Error = ExchangeError;

    fn try_from(value: TradeFrame) -> Result<Self, Self::Error> {
        match value {
            TradeFrame::AggTrade(trade) => Ok(exc_core::types::Trade {
                ts: crate::types::adaptations::from_timestamp(trade.trade_timestamp)?,
                price: trade.price.normalize(),
                size: trade.size.normalize(),
                buy: !trade.buy_maker,
            }),
            TradeFrame::Trade(trade) => Ok(exc_core::types::Trade {
                ts: crate::types::adaptations::from_timestamp(trade.trade_timestamp)?,
                price: trade.price.normalize(),
                size: trade.size.normalize(),
                buy: trade.is_taker_buy(),
            }),
        }
    }
}

/// Depth frame.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub enum DepthFrame {
    /// Book ticker.
    BookTicker(BookTicker),
    /// Depth.
    Depth(depth::Depth),
}

impl TryFrom<StreamFrame> for DepthFrame {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        match frame.data {
            StreamFrameKind::BookTicker(t) => Ok(Self::BookTicker(t)),
            StreamFrameKind::Depth(t) => Ok(Self::Depth(t)),
            _ => Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}"))),
        }
    }
}

impl TryFrom<DepthFrame> for exc_core::types::BidAsk {
    type Error = ExchangeError;

    fn try_from(value: DepthFrame) -> Result<Self, Self::Error> {
        match value {
            DepthFrame::BookTicker(t) => Ok(exc_core::types::BidAsk {
                ts: t
                    .trade_timestamp
                    .map(crate::types::adaptations::from_timestamp)
                    .transpose()?
                    .unwrap_or_else(time::OffsetDateTime::now_utc),
                bid: Some((t.bid.normalize(), t.bid_size.normalize())),
                ask: Some((t.ask.normalize(), t.ask_size.normalize())),
            }),
            DepthFrame::Depth(t) => Ok(exc_core::types::BidAsk {
                ts: crate::types::adaptations::from_timestamp(t.trade_timestamp)?,
                bid: t.bids.first().map(|b| (b.0.normalize(), b.1.normalize())),
                ask: t.asks.first().map(|a| (a.0.normalize(), a.1.normalize())),
            }),
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
                .and_then(ServerFrame::health)
                .map(|f| stream::iter(f.break_down().into_iter().map(Ok)));
            future::ready(f)
        })
        .try_flatten()
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
