use exc_core::Str;
use serde::Serialize;

/// Order
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Instrument Id.
    pub inst_id: Str,
    /// Order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ord_id: Option<Str>,
    /// Clinet order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<Str>,
}
