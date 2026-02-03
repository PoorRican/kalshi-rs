use thiserror::Error;

#[derive(Debug, Error)]
pub enum KalshiError {
    #[error("authentication required: {0}")]
    AuthRequired(&'static str),

    #[error("invalid parameters: {0}")]
    InvalidParams(String),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("header error: {0}")]
    Header(String),

    #[error("websocket error: {0}")]
    Ws(String),
}

