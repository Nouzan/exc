use super::Args;
use crate::error::OkxError;
use crate::key::{OkxKey as Key, Signature};
use exc_core::types::trading::{OrderKind, Place, PlaceOrderOptions};
use exc_core::types::TimeInForce;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use tokio_tungstenite::tungstenite::Message;

/// Okx websocket operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Op {
    /// Subscribe.
    Subscribe,
    /// Unsubsribe.
    Unsubscribe,
    /// Login.
    Login,
    /// Order.
    Order,
    /// Cancel order.
    CancelOrder,
}

/// Okx websocket request messagee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRequestMessage {
    /// Id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Operation.
    pub op: Op,
    /// Arguments.
    #[serde(default)]
    pub args: Vec<Args>,
}

impl WsRequestMessage {
    /// Convert into a websocket message.
    pub fn to_websocket(&self) -> Result<Message, OkxError> {
        let text = serde_json::to_string(&self)?;
        Ok(Message::Text(text))
    }
}

/// Okx websocket request.
#[derive(Debug, Clone)]
pub enum WsRequest {
    /// Subscribe.
    Subscribe(Args),
    /// Unsubscribe.
    Unsubscribe(Args),
    /// Login.
    Login(Args),
    /// Order.
    Order(String, Args),
    /// Cancel order.
    CancelOrder(String, Args),
}

impl fmt::Display for WsRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subscribe(args) => {
                write!(f, "sub:{args}")
            }
            Self::Unsubscribe(args) => {
                write!(f, "unsub:{args}")
            }
            Self::Login(_args) => {
                write!(f, "login")
            }
            Self::Order(id, args) => {
                write!(f, "order:{id}:{args}")
            }
            Self::CancelOrder(id, args) => {
                write!(f, "cancel-order:{id}:{args}")
            }
        }
    }
}

impl WsRequest {
    /// Subscribe tickers.
    pub fn subscribe_tickers(inst: &str) -> Self {
        Self::Subscribe(Args(BTreeMap::from([
            ("channel".to_string(), "tickers".to_string()),
            ("instId".to_string(), inst.to_string()),
        ])))
    }

    /// Unsubscribe tickers.
    pub fn unsubscribe_tickers(inst: &str) -> Self {
        Self::Unsubscribe(Args(BTreeMap::from([
            ("channel".to_string(), "tickers".to_string()),
            ("instId".to_string(), inst.to_string()),
        ])))
    }

    /// Login request.
    pub(crate) fn login(key: Key, signature: Signature) -> Self {
        Self::Login(Args(BTreeMap::from([
            ("apiKey".to_string(), key.apikey),
            ("passphrase".to_string(), key.passphrase),
            ("timestamp".to_string(), signature.timestamp),
            ("sign".to_string(), signature.signature),
        ])))
    }

    /// Order request.
    pub(crate) fn order(place: &Place, opts: &PlaceOrderOptions) -> Self {
        let inst = opts.instrument();
        let custom = opts.custom();
        let size = place.size.abs();
        let side = if place.size.is_sign_negative() {
            "sell"
        } else {
            "buy"
        };
        let mut map = BTreeMap::from([
            ("instId".to_string(), inst.to_string()),
            (
                "tdMode".to_string(),
                custom
                    .get("tdMode")
                    .map(|s| s.as_str())
                    .unwrap_or("cross")
                    .to_string(),
            ),
            ("side".to_string(), side.to_string()),
            ("posSide".to_string(), "net".to_string()),
            ("sz".to_string(), size.to_string()),
        ]);
        if let Some(margin) = opts.margin() {
            map.insert("ccy".to_string(), margin.to_string());
        }
        match place.kind {
            OrderKind::Market => {
                map.insert("ordType".to_string(), "market".to_string());
                if inst.split('-').count() == 2 {
                    // spot-market
                    map.insert("tgtCcy".to_string(), "base_ccy".to_string());
                }
            }
            OrderKind::Limit(price, tif) => {
                map.insert("px".to_string(), price.to_string());
                let t = match tif {
                    TimeInForce::GoodTilCancelled => "limit",
                    TimeInForce::FillOrKill => "fok",
                    TimeInForce::ImmediateOrCancel => "ioc",
                };
                map.insert("ordType".to_string(), t.to_string());
            }
            OrderKind::PostOnly(price) => {
                map.insert("px".to_string(), price.to_string());
                map.insert("ordType".to_string(), "post_only".to_string());
            }
        }
        Self::Order(format!("{:x}", uuid::Uuid::new_v4().as_u128()), Args(map))
    }

    /// Cancel order request.
    pub(crate) fn cancel_order(inst: &str, id: &str) -> Self {
        Self::CancelOrder(
            format!("{:x}", uuid::Uuid::new_v4().as_u128()),
            Args(BTreeMap::from([
                ("instId".to_string(), inst.to_string()),
                ("ordId".to_string(), id.to_string()),
            ])),
        )
    }
}

impl From<WsRequest> for WsRequestMessage {
    fn from(req: WsRequest) -> Self {
        match req {
            WsRequest::Subscribe(args) => Self {
                id: None,
                op: Op::Subscribe,
                args: vec![args],
            },
            WsRequest::Unsubscribe(args) => Self {
                id: None,
                op: Op::Unsubscribe,
                args: vec![args],
            },
            WsRequest::Login(args) => Self {
                id: None,
                op: Op::Login,
                args: vec![args],
            },
            WsRequest::Order(id, args) => Self {
                id: Some(id),
                op: Op::Order,
                args: vec![args],
            },
            WsRequest::CancelOrder(id, args) => Self {
                id: Some(id),
                op: Op::CancelOrder,
                args: vec![args],
            },
        }
    }
}
