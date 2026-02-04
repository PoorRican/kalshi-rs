use crate::error::KalshiError;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::rngs::OsRng;
use rsa::{pkcs1::DecodeRsaPrivateKey, pkcs8::DecodePrivateKey, RsaPrivateKey};
use rsa::pss::SigningKey;
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct KalshiAuth {
    pub key_id: String,
    private_key: RsaPrivateKey,
}

/// Convenience container for the three auth headers.
#[derive(Debug, Clone)]
pub struct KalshiAuthHeaders {
    pub key: String,
    pub timestamp_ms: String,
    pub signature: String,
}

impl KalshiAuth {
    /// Load a `.key` PEM file (Kalshi UI downloads a private key as `.key`) :contentReference[oaicite:16]{index=16}
    pub fn from_pem_file(key_id: impl Into<String>, pem_path: impl AsRef<std::path::Path>) -> Result<Self, KalshiError> {
        let pem = std::fs::read_to_string(pem_path)?;
        Self::from_pem_str(key_id, &pem)
    }

    /// Load from a PEM string (supports PKCS#8 and PKCS#1).
    pub fn from_pem_str(key_id: impl Into<String>, pem: &str) -> Result<Self, KalshiError> {
        let key_id = key_id.into();

        let private_key = RsaPrivateKey::from_pkcs8_pem(pem)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(pem))
            .map_err(|e| KalshiError::Crypto(e.to_string()))?;

        Ok(Self { key_id, private_key })
    }

    /// Milliseconds since UNIX epoch, as required by Kalshi auth headers. :contentReference[oaicite:17]{index=17}
    pub fn now_timestamp_ms() -> String {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_millis();
        ms.to_string()
    }

    /// Create signature for a request:
    /// message = timestamp + METHOD + path_without_query
    /// signature = RSA-PSS(SHA256), base64 encoded :contentReference[oaicite:18]{index=18}
    pub fn sign(&self, timestamp_ms: &str, method: &str, path: &str) -> Result<String, KalshiError> {
        let message = Self::signing_message(timestamp_ms, method, path);
        let message_bytes = message.as_bytes();

        // RSA-PSS is randomized; use OS RNG.
        let mut rng = OsRng;

        // PSS with SHA256 (salt length = digest length) per docs :contentReference[oaicite:19]{index=19}
        let signing_key = SigningKey::<Sha256>::new(self.private_key.clone());
        let signature = signing_key
            .sign_with_rng(&mut rng, message_bytes);

        Ok(STANDARD.encode(signature.to_bytes()))
    }

    /// Build the canonical signing message (timestamp + METHOD + path_without_query).
    pub fn signing_message(timestamp_ms: &str, method: &str, path: &str) -> String {
        let method = method.to_uppercase();
        let path_without_query = path.split('?').next().unwrap_or(path);
        format!("{timestamp_ms}{method}{path_without_query}")
    }

    /// Build the three headers required by Kalshi authenticated endpoints. :contentReference[oaicite:20]{index=20}
    pub fn build_headers(&self, method: &str, path: &str) -> Result<KalshiAuthHeaders, KalshiError> {
        let timestamp_ms = Self::now_timestamp_ms();
        let signature = self.sign(&timestamp_ms, method, path)?;

        Ok(KalshiAuthHeaders {
            key: self.key_id.clone(),
            timestamp_ms,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::KalshiAuth;

    #[test]
    fn signing_message_strips_query() {
        let msg = KalshiAuth::signing_message(
            "1700000000000",
            "get",
            "/trade-api/v2/markets?limit=10&cursor=abc",
        );
        assert_eq!(msg, "1700000000000GET/trade-api/v2/markets");
    }
}
