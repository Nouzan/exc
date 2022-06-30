use serde::Deserialize;

use crate::websocket::error::WsError;

use super::{Name, Nameable, StreamFrame, StreamFrameKind};

/// Account events.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "e", rename_all = "camelCase")]
pub enum AccountEvent {
    /// Listen key expired.
    ListenKeyExpired {
        /// Event timestamp.
        #[serde(rename = "E")]
        ts: i64,
    },
    /// Order trade update.
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    OrderTradeUpdate {
        /// Event timestamp.
        #[serde(rename = "E")]
        event_ts: i64,
        /// Trade timestamp.
        #[serde(rename = "T")]
        trade_ts: i64,
        /// Order.
        #[serde(rename = "o")]
        order: serde_json::Value,
    },
}

impl Nameable for AccountEvent {
    fn to_name(&self) -> Name {
        match self {
            Self::ListenKeyExpired { .. } => Name::listen_key_expired(),
            Self::OrderTradeUpdate { .. } => Name::order_trade_update(),
        }
    }
}

impl TryFrom<StreamFrame> for AccountEvent {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        if let StreamFrameKind::AccountEvent(e) = frame.data {
            Ok(e)
        } else {
            Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}")))
        }
    }
}
