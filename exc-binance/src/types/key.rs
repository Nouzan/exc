use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;
use time::OffsetDateTime;

/// Sign error.
#[derive(Debug, Error)]
pub enum SignError {
    /// Invalid length.
    #[error("invalid length of secretkey")]
    InvalidLength,
    /// Urlencoded.
    #[error("urlencoded: {0}")]
    Urlencoded(#[from] serde_urlencoded::ser::Error),
}

type HmacSha256 = Hmac<Sha256>;

/// Binance API Key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceKey {
    /// Apikey.
    pub apikey: String,
    /// Secretkey.
    pub secretkey: String,
}

impl BinanceKey {
    /// Sign.
    pub fn sign<T: Serialize>(&self, params: T) -> Result<SignedParams<T>, SignError> {
        SigningParams::now(params).signed(&self)
    }
}

/// Signing params.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SigningParams<T> {
    #[serde(flatten)]
    params: T,
    #[serde(rename = "recvWindow")]
    recv_window: i64,
    timestamp: i64,
}

impl<T> SigningParams<T> {
    fn with_timestamp(params: T, timestamp: i64) -> Self {
        Self {
            params,
            recv_window: 5000,
            timestamp,
        }
    }

    /// Sign the given params now.
    pub fn now(params: T) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;
        Self::with_timestamp(params, now as i64)
    }
}

/// Signed params.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignedParams<T> {
    #[serde(flatten)]
    signing: SigningParams<T>,
    signature: String,
}

impl<T: Serialize> SigningParams<T> {
    /// Get signed params.
    pub fn signed(self, key: &BinanceKey) -> Result<SignedParams<T>, SignError> {
        let raw = serde_urlencoded::to_string(&self)?;
        tracing::debug!("raw string to sign: {}", raw);
        let mut mac = HmacSha256::new_from_slice(key.secretkey.as_bytes())
            .map_err(|_| SignError::InvalidLength)?;
        mac.update(raw.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        Ok(SignedParams {
            signing: self,
            signature,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Serialize)]
    struct Params {
        symbol: String,
        side: String,
        #[serde(rename = "type")]
        kind: String,
        #[serde(rename = "timeInForce")]
        time_in_force: String,
        quantity: i64,
        price: f64,
    }

    #[test]
    fn test_hmac_sha256() -> anyhow::Result<()> {
        let key = "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j";
        let raw = "asset=ETH&address=0x6915f16f8791d0a1cc2bf47c13a6b2a92000504b&amount=1&recvWindow=5000&name=test&timestamp=1510903211000";
        let mut mac = HmacSha256::new_from_slice(key.as_bytes())?;
        mac.update(raw.as_bytes());
        let encoded = hex::encode(mac.finalize().into_bytes());
        println!("{}", encoded);
        assert_eq!(
            encoded,
            "157fb937ec848b5f802daa4d9f62bea08becbf4f311203bda2bd34cd9853e320"
        );
        Ok(())
    }

    #[test]
    fn test_signature() -> anyhow::Result<()> {
        let key = BinanceKey {
            apikey: "".to_string(),
            secretkey: "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j"
                .to_string(),
        };
        let params = Params {
            symbol: "LTCBTC".to_string(),
            side: "BUY".to_string(),
            kind: "LIMIT".to_string(),
            time_in_force: "GTC".to_string(),
            quantity: 1,
            price: 0.1,
        };
        println!("raw={}", serde_qs::to_string(&params)?);
        let signed = SigningParams::with_timestamp(params, 1499827319559).signed(&key)?;
        let target = serde_json::json!({
            "symbol": "LTCBTC",
            "side": "BUY",
            "type": "LIMIT",
            "timeInForce": "GTC",
            "quantity": 1,
            "price": 0.1,
            "recvWindow": 5000,
            "timestamp": 1499827319559_i64,
            "signature": "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71",
        });
        assert_eq!(serde_json::to_value(&signed)?, target);
        Ok(())
    }
}
