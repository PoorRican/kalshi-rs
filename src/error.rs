use crate::types::ErrorResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KalshiError {
    #[error("authentication required: {0}")]
    AuthRequired(&'static str),

    #[error("invalid parameters: {0}")]
    InvalidParams(String),

    #[error("http error {status}")]
    Http {
        status: reqwest::StatusCode,
        api_error: Option<ErrorResponse>,
        raw_body: String,
        request_id: Option<String>,
    },

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("parse error ({context}): {reason}")]
    Parse {
        context: String,
        reason: String,
        raw: Vec<u8>,
        #[source]
        source: Option<serde_json::Error>,
    },

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("header error: {0}")]
    Header(String),

    #[error("websocket error: {0}")]
    Ws(String),
}

impl KalshiError {
    pub(crate) fn parse_json(
        context: impl Into<String>,
        raw: impl AsRef<[u8]>,
        source: serde_json::Error,
    ) -> Self {
        let reason = source.to_string();
        Self::Parse {
            context: context.into(),
            reason,
            raw: raw.as_ref().to_vec(),
            source: Some(source),
        }
    }

    pub(crate) fn parse_reason(
        context: impl Into<String>,
        raw: impl AsRef<[u8]>,
        reason: impl Into<String>,
    ) -> Self {
        Self::Parse {
            context: context.into(),
            reason: reason.into(),
            raw: raw.as_ref().to_vec(),
            source: None,
        }
    }

    pub fn parse_context(&self) -> Option<&str> {
        match self {
            Self::Parse { context, .. } => Some(context),
            _ => None,
        }
    }

    pub fn parse_error_reason(&self) -> Option<&str> {
        match self {
            Self::Parse { reason, .. } => Some(reason),
            _ => None,
        }
    }

    pub fn parse_raw_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Parse { raw, .. } => Some(raw),
            _ => None,
        }
    }
}
