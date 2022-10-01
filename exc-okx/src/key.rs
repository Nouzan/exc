use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;
use time::{error::Format, format_description::well_known::Rfc3339, OffsetDateTime};

type HmacSha256 = Hmac<Sha256>;

/// Error type for signing.
#[derive(Debug, Error)]
pub enum SignError {
    /// Format timestamp error.
    #[error("format timestamp error: {0}")]
    FormatTimestamp(#[from] Format),

    /// Convert timetsamp error.
    #[error("convert timestamp error: {0}")]
    ConvertTimestamp(#[from] time::error::ComponentRange),

    /// SecretKey length error.
    #[error("secretkey length error")]
    SecretKeyLength,
}

/// The APIKey definition of OKX.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OkxKey {
    /// APIKey.
    pub apikey: String,
    /// SecretKey.
    pub secretkey: String,
    /// Passphrase.
    pub passphrase: String,
}

/// Signature
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Signature {
    /// Signature.
    #[serde(rename = "sign")]
    pub signature: String,

    /// Timestamp.
    pub timestamp: String,
}

impl OkxKey {
    /// Create a new [`Key`].
    pub fn new(apikey: &str, secretkey: &str, passphrase: &str) -> Self {
        Self {
            apikey: apikey.to_string(),
            secretkey: secretkey.to_string(),
            passphrase: passphrase.to_string(),
        }
    }

    /// Sign with this [`Key`].
    pub fn sign(
        &self,
        method: &str,
        uri: &str,
        timestamp: OffsetDateTime,
        use_unix_timestamp: bool,
    ) -> Result<Signature, SignError> {
        let secret = self.secretkey.as_str();
        let timestamp = timestamp.replace_millisecond(timestamp.millisecond())?;
        let timestamp = if use_unix_timestamp {
            timestamp.unix_timestamp().to_string()
        } else {
            timestamp.format(&Rfc3339)?
        };
        let raw_sign = timestamp.clone() + method + uri;
        tracing::debug!("message to sign: {}", raw_sign);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| SignError::SecretKeyLength)?;
        mac.update(raw_sign.as_bytes());

        Ok(Signature {
            signature: base64::encode(mac.finalize().into_bytes()),
            timestamp,
        })
    }

    /// Sign now.
    pub fn sign_now(
        &self,
        method: &str,
        uri: &str,
        use_unix_timestamp: bool,
    ) -> Result<Signature, SignError> {
        self.sign(method, uri, OffsetDateTime::now_utc(), use_unix_timestamp)
    }
}
