use crate::KalshiError;
use crate::types::ErrorResponse;
use rand::random;
use reqwest::header::{HeaderMap, RETRY_AFTER};
use reqwest::{Method, StatusCode};
use std::time::SystemTime;
use tokio::time::Duration;

/// HTTP retry policy for transient REST failures.
///
/// # Default
///
/// - Retries enabled for idempotent methods (`GET`, `DELETE`)
/// - Retries disabled for non-idempotent methods (`POST`, `PUT`, `PATCH`)
/// - `max_retries = 3` (attempts after the initial request)
/// - Exponential backoff with jitter
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retries after the initial attempt.
    pub max_retries: u32,
    /// Initial backoff delay.
    pub base_delay: Duration,
    /// Maximum backoff delay.
    pub max_delay: Duration,
    /// Jitter factor in range `[0.0, 1.0]`.
    pub jitter: f64,
    /// Whether to retry non-idempotent methods.
    pub retry_non_idempotent: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(5),
            jitter: 0.2,
            retry_non_idempotent: false,
        }
    }
}

impl RetryConfig {
    pub(crate) fn allows_method(&self, method: &Method) -> bool {
        matches!(*method, Method::GET | Method::DELETE)
            || (self.retry_non_idempotent
                && matches!(*method, Method::POST | Method::PUT | Method::PATCH))
    }

    pub(crate) fn backoff_delay(&self, retry_number: u32) -> Duration {
        let exp = 2f64.powi(retry_number.saturating_sub(1) as i32);
        let mut delay = self.base_delay.mul_f64(exp);
        if delay > self.max_delay {
            delay = self.max_delay;
        }

        let jitter = self.jitter.clamp(0.0, 1.0);
        if jitter > 0.0 {
            let factor = 1.0 - jitter + random::<f64>() * (2.0 * jitter);
            delay = delay.mul_f64(factor);
        }
        delay
    }
}

pub(crate) fn build_http_error(
    status: reqwest::StatusCode,
    bytes: &[u8],
    request_id: Option<String>,
) -> KalshiError {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct WrappedErrorBody {
        error: ErrorResponse,
    }

    let raw_body = String::from_utf8_lossy(bytes).to_string();
    let normalize = |error: ErrorResponse| {
        if error.code.is_some() || error.message.is_some() || error.details.is_some() {
            Some(error)
        } else {
            None
        }
    };
    let api_error = serde_json::from_slice::<WrappedErrorBody>(bytes)
        .ok()
        .and_then(|wrapped| normalize(wrapped.error))
        .or_else(|| {
            serde_json::from_slice::<ErrorResponse>(bytes)
                .ok()
                .and_then(normalize)
        });
    KalshiError::Http {
        status,
        api_error,
        raw_body,
        request_id,
    }
}

pub(crate) fn retryable_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::REQUEST_TIMEOUT
            | StatusCode::TOO_EARLY
            | StatusCode::TOO_MANY_REQUESTS
            | StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

pub(crate) fn retryable_reqwest_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect() || err.is_request()
}

pub(crate) fn retry_after_delay(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?;
    let text = value.to_str().ok()?.trim();

    if let Ok(seconds) = text.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    let ts = httpdate::parse_http_date(text).ok()?;
    let now = SystemTime::now();
    let delta = ts.duration_since(now).ok()?;
    Some(delta)
}
