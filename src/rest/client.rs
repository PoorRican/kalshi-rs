use crate::rest::events::GetEventForecastPercentileHistoryParams;
use crate::rest::rate_limit::{RateLimitConfig, RateLimitTier, RateLimiter, rate_limit_kind};
use crate::rest::retry::{
    RetryConfig, build_http_error, retry_after_delay, retryable_reqwest_error, retryable_status,
};
use crate::rest::structured_targets::GetStructuredTargetsParams;
use crate::{KalshiAuth, KalshiEnvironment, KalshiError, REST_PREFIX};

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method, Proxy, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use url::Url;

/// Builder for [`KalshiRestClient`] with transport and retry customization.
#[derive(Debug, Clone)]
pub struct KalshiRestClientBuilder {
    env: KalshiEnvironment,
    auth: Option<KalshiAuth>,
    rate_limit_config: RateLimitConfig,
    retry_config: RetryConfig,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    user_agent: Option<String>,
    default_headers: Option<HeaderMap>,
    proxy: Option<Proxy>,
    proxy_error: Option<String>,
    http_client: Option<Client>,
}

impl KalshiRestClientBuilder {
    fn new(env: KalshiEnvironment) -> Self {
        Self {
            env,
            auth: None,
            rate_limit_config: RateLimitConfig::default(),
            retry_config: RetryConfig::default(),
            timeout: None,
            connect_timeout: None,
            user_agent: None,
            default_headers: None,
            proxy: None,
            proxy_error: None,
            http_client: None,
        }
    }

    pub fn with_auth(mut self, auth: KalshiAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn with_rate_limit_config(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit_config = config;
        self
    }

    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn with_default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = Some(headers);
        self
    }

    /// Configure an HTTP proxy for the internally-built `reqwest::Client`.
    ///
    /// Accepts either a concrete [`reqwest::Proxy`] or a `Result<Proxy, reqwest::Error>`
    /// from helper constructors like [`reqwest::Proxy::all`].
    pub fn with_proxy(mut self, proxy: impl Into<Result<Proxy, reqwest::Error>>) -> Self {
        match proxy.into() {
            Ok(proxy) => {
                self.proxy = Some(proxy);
                self.proxy_error = None;
            }
            Err(err) => {
                self.proxy = None;
                self.proxy_error = Some(err.to_string());
            }
        }
        self
    }

    pub fn with_http_client(mut self, client: Client) -> Self {
        self.http_client = Some(client);
        self
    }

    pub fn build(self) -> Result<KalshiRestClient, KalshiError> {
        let http = if let Some(client) = self.http_client {
            client
        } else {
            if let Some(proxy_error) = self.proxy_error {
                return Err(KalshiError::InvalidParams(format!(
                    "invalid proxy configuration: {proxy_error}"
                )));
            }

            let mut builder = Client::builder();
            if let Some(timeout) = self.timeout {
                builder = builder.timeout(timeout);
            }
            if let Some(timeout) = self.connect_timeout {
                builder = builder.connect_timeout(timeout);
            }
            if let Some(user_agent) = self.user_agent {
                builder = builder.user_agent(user_agent);
            }
            if let Some(headers) = self.default_headers {
                builder = builder.default_headers(headers);
            }
            if let Some(proxy) = self.proxy {
                builder = builder.proxy(proxy);
            }
            builder.build()?
        };

        Ok(KalshiRestClient {
            http,
            rest_origin: self.env.rest_origin,
            auth: self.auth,
            rate_limiter: Arc::new(RateLimiter::new(self.rate_limit_config)),
            retry_config: self.retry_config,
        })
    }
}

/// Async HTTP client for the Kalshi REST API.
///
/// Provides methods for every public and authenticated endpoint, plus
/// pagination helpers ([`CursorPager`] and `stream_*` methods).
///
/// # Construction
///
/// ```no_run
/// use kalshi_fast::{KalshiAuth, KalshiEnvironment, KalshiRestClient};
///
/// # fn run() -> Result<(), kalshi_fast::KalshiError> {
/// let client = KalshiRestClient::new(KalshiEnvironment::demo())
///     .with_auth(KalshiAuth::from_pem_file("key-id", "key.pem")?)
///     .with_rate_limit_config(kalshi_fast::RateLimitConfig {
///         read_rps: 10,
///         write_rps: 5,
///     });
/// # Ok(())
/// # }
/// ```
///
/// # Public vs Authenticated Endpoints
///
/// | Category | Prefix | Auth required |
/// |----------|--------|---------------|
/// | Markets, events, trades, series | `/markets`, `/events`, `/series` | No |
/// | Exchange status / schedule | `/exchange` | No |
/// | Portfolio (balance, positions, orders, fills) | `/portfolio` | Yes |
/// | Account limits | `/account` | Yes |
///
/// Calling an authenticated endpoint without [`with_auth`](Self::with_auth)
/// returns [`KalshiError::AuthRequired`](crate::KalshiError::AuthRequired).
#[derive(Debug, Clone)]
pub struct KalshiRestClient {
    http: Client,
    rest_origin: Url,
    auth: Option<KalshiAuth>,
    rate_limiter: Arc<RateLimiter>,
    retry_config: RetryConfig,
}

impl KalshiRestClient {
    /// Start a configurable client builder.
    pub fn builder(env: KalshiEnvironment) -> KalshiRestClientBuilder {
        KalshiRestClientBuilder::new(env)
    }

    /// Create a new client targeting the given environment (demo or production).
    ///
    /// The client starts **unauthenticated** with the Basic rate-limit tier.
    /// Chain [`with_auth`](Self::with_auth) and/or
    /// [`with_rate_limit_config`](Self::with_rate_limit_config) as needed.
    pub fn new(env: KalshiEnvironment) -> Self {
        Self::builder(env)
            .build()
            .expect("default rest client builder should not fail")
    }

    /// Attach auth so you can call authenticated endpoints.
    pub fn with_auth(mut self, auth: KalshiAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Override rate limits with a known tier.
    pub fn with_rate_limit_tier(mut self, tier: RateLimitTier) -> Self {
        self.rate_limiter = Arc::new(RateLimiter::new(tier.config()));
        self
    }

    /// Override rate limits with a custom configuration.
    pub fn with_rate_limit_config(mut self, config: RateLimitConfig) -> Self {
        self.rate_limiter = Arc::new(RateLimiter::new(config));
        self
    }

    /// Override retry policy.
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    pub(crate) fn full_path(endpoint_path: &str) -> String {
        // endpoint_path must begin with "/", e.g. "/markets"
        format!("{REST_PREFIX}{endpoint_path}")
    }

    pub(crate) fn build_url(&self, full_path: &str) -> Result<Url, KalshiError> {
        Ok(self.rest_origin.join(full_path)?)
    }

    pub(crate) fn insert_auth_headers(
        headers: &mut HeaderMap,
        auth: &KalshiAuth,
        method: &Method,
        path_without_query: &str,
    ) -> Result<(), KalshiError> {
        let h = auth.build_headers(method.as_str(), path_without_query)?;

        headers.insert(
            HeaderName::from_static("kalshi-access-key"),
            HeaderValue::from_str(&h.key).map_err(|e| KalshiError::Header(e.to_string()))?,
        );
        headers.insert(
            HeaderName::from_static("kalshi-access-timestamp"),
            HeaderValue::from_str(&h.timestamp_ms)
                .map_err(|e| KalshiError::Header(e.to_string()))?,
        );
        headers.insert(
            HeaderName::from_static("kalshi-access-signature"),
            HeaderValue::from_str(&h.signature).map_err(|e| KalshiError::Header(e.to_string()))?,
        );

        Ok(())
    }

    pub(crate) async fn send<Q, B, T>(
        &self,
        method: Method,
        full_path: &str,
        query: Option<&Q>,
        body: Option<&B>,
        require_auth: bool,
    ) -> Result<T, KalshiError>
    where
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = self.build_url(full_path)?;
        let auth = if require_auth {
            Some(
                self.auth
                    .as_ref()
                    .ok_or(KalshiError::AuthRequired("REST endpoint"))?,
            )
        } else {
            None
        };
        let body_bytes = match body {
            Some(value) => Some(serde_json::to_vec(value)?),
            None => None,
        };

        let mut retry_number: u32 = 0;

        loop {
            let mut headers = HeaderMap::new();
            if let Some(auth) = auth {
                // IMPORTANT: sign the path without query parameters.
                Self::insert_auth_headers(&mut headers, auth, &method, full_path)?;
            }

            self.rate_limiter.wait(rate_limit_kind(&method)).await;

            let mut req = self
                .http
                .request(method.clone(), url.clone())
                .headers(headers);

            if let Some(q) = query {
                req = req.query(q);
            }
            if let Some(body) = &body_bytes {
                req = req
                    .header(CONTENT_TYPE, "application/json")
                    .body(body.clone());
            }

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let headers = resp.headers().clone();
                    let request_id = headers
                        .get("x-request-id")
                        .or_else(|| headers.get("request-id"))
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());

                    let retry_after = if status == StatusCode::TOO_MANY_REQUESTS {
                        retry_after_delay(&headers)
                    } else {
                        None
                    };
                    let bytes = resp.bytes().await?;

                    if status.is_success() {
                        let body_bytes = if bytes.is_empty() {
                            b"{}"
                        } else {
                            bytes.as_ref()
                        };
                        return serde_json::from_slice::<T>(body_bytes).map_err(|source| {
                            KalshiError::parse_json(
                                format!("REST {} {}", method, full_path),
                                body_bytes,
                                source,
                            )
                        });
                    }

                    let should_retry = retry_number < self.retry_config.max_retries
                        && self.retry_config.allows_method(&method)
                        && retryable_status(status);

                    if should_retry {
                        retry_number = retry_number.saturating_add(1);
                        let delay = retry_after
                            .unwrap_or_else(|| self.retry_config.backoff_delay(retry_number));
                        if !delay.is_zero() {
                            sleep(delay).await;
                        }
                        continue;
                    }

                    return Err(build_http_error(status, &bytes, request_id));
                }
                Err(err) => {
                    let should_retry = retry_number < self.retry_config.max_retries
                        && self.retry_config.allows_method(&method)
                        && retryable_reqwest_error(&err);
                    if should_retry {
                        retry_number = retry_number.saturating_add(1);
                        let delay = self.retry_config.backoff_delay(retry_number);
                        if !delay.is_zero() {
                            sleep(delay).await;
                        }
                        continue;
                    }
                    return Err(err.into());
                }
            }
        }
    }

    pub(crate) fn event_forecast_percentile_history_query(
        params: &GetEventForecastPercentileHistoryParams,
    ) -> Vec<(String, String)> {
        let mut query = Vec::with_capacity(params.percentiles.len() + 3);
        for percentile in &params.percentiles {
            query.push(("percentiles".to_string(), percentile.to_string()));
        }
        query.push(("start_ts".to_string(), params.start_ts.to_string()));
        query.push(("end_ts".to_string(), params.end_ts.to_string()));
        query.push((
            "period_interval".to_string(),
            params.period_interval.to_string(),
        ));
        query
    }

    pub(crate) fn structured_targets_query(
        params: &GetStructuredTargetsParams,
    ) -> Vec<(String, String)> {
        let mut query = Vec::new();

        if let Some(ids) = &params.ids {
            for id in ids {
                query.push(("ids".to_string(), id.clone()));
            }
        }
        if let Some(target_type) = &params.target_type {
            query.push(("type".to_string(), target_type.clone()));
        }
        if let Some(competition) = &params.competition {
            query.push(("competition".to_string(), competition.clone()));
        }
        if let Some(page_size) = params.page_size {
            query.push(("page_size".to_string(), page_size.to_string()));
        }
        if let Some(cursor) = &params.cursor {
            query.push(("cursor".to_string(), cursor.clone()));
        }

        query
    }
    // -----------------------------------------------
    // Additional public endpoints
    // -----------------------------------------------

    // -----------------------------------------------
    // Generic pagination
    // -----------------------------------------------

    /// Eagerly fetch **all** pages from a cursor-paginated callback into a single `Vec`.
    ///
    /// Prefer [`CursorPager`] or the `stream_*` methods when you don't need every item in
    /// memory at once.
    pub async fn paginate_cursor<T, F, Fut>(
        &self,
        mut cursor: Option<String>,
        mut fetch: F,
    ) -> Result<Vec<T>, KalshiError>
    where
        F: FnMut(Option<String>) -> Fut,
        Fut: std::future::Future<Output = Result<(Vec<T>, Option<String>), KalshiError>>,
    {
        let mut items = Vec::new();
        loop {
            let (page_items, next) = fetch(cursor.clone()).await?;
            items.extend(page_items);
            cursor = next.filter(|c| !c.is_empty());
            if cursor.is_none() {
                break;
            }
        }
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::pagination::{CursorPager, stream_items};
    use crate::rest::rate_limit::RateLimitKind;
    use futures::stream::TryStreamExt;
    use reqwest::Method;
    use reqwest::StatusCode;
    use serde_json::json;
    use std::collections::VecDeque;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::time::{Duration, Instant, timeout};
    use url::Url;

    #[derive(Clone)]
    struct TestHttpResponse {
        status: StatusCode,
        headers: Vec<(String, String)>,
        body: String,
    }

    impl TestHttpResponse {
        fn new(status: StatusCode, body: impl Into<String>) -> Self {
            Self {
                status,
                headers: Vec::new(),
                body: body.into(),
            }
        }

        fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.headers.push((key.into(), value.into()));
            self
        }
    }

    fn header_end(buf: &[u8]) -> Option<usize> {
        buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
    }

    async fn read_http_request(stream: &mut tokio::net::TcpStream) -> std::io::Result<()> {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 2048];
        let mut required_body_len: Option<usize> = None;
        let mut header_len: Option<usize> = None;

        loop {
            let n = stream.read(&mut chunk).await?;
            if n == 0 {
                return Ok(());
            }
            buffer.extend_from_slice(&chunk[..n]);

            if header_len.is_none()
                && let Some(end) = header_end(&buffer)
            {
                header_len = Some(end);
                let headers = String::from_utf8_lossy(&buffer[..end]).to_ascii_lowercase();
                let content_length = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .and_then(|value| value.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                required_body_len = Some(content_length);
            }

            if let (Some(header_len), Some(required_body_len)) = (header_len, required_body_len) {
                let body_len = buffer.len().saturating_sub(header_len);
                if body_len >= required_body_len {
                    return Ok(());
                }
            }
        }
    }

    async fn spawn_http_sequence_server(
        responses: Vec<TestHttpResponse>,
    ) -> (
        Url,
        Arc<AtomicUsize>,
        tokio::task::JoinHandle<std::io::Result<()>>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_ref = Arc::clone(&hits);

        let task = tokio::spawn(async move {
            for response in responses {
                let (mut stream, _) = listener.accept().await?;
                read_http_request(&mut stream).await?;
                hits_ref.fetch_add(1, Ordering::Relaxed);

                let reason = response.status.canonical_reason().unwrap_or("Unknown");
                let mut reply = format!(
                    "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n",
                    response.status.as_u16(),
                    reason,
                    response.body.len()
                );
                for (key, value) in response.headers {
                    reply.push_str(&format!("{key}: {value}\r\n"));
                }
                reply.push_str("\r\n");
                reply.push_str(&response.body);

                stream.write_all(reply.as_bytes()).await?;
                stream.flush().await?;
            }
            Ok(())
        });

        (
            Url::parse(&format!("http://{addr}")).expect("url"),
            hits,
            task,
        )
    }

    fn test_env(rest_origin: Url) -> KalshiEnvironment {
        KalshiEnvironment {
            rest_origin,
            ws_url: "ws://127.0.0.1/".to_string(),
        }
    }

    #[test]
    fn http_error_parses_json_body() {
        let body = br#"{"code":"rate_limit","message":"too fast"}"#;
        let err = build_http_error(
            StatusCode::TOO_MANY_REQUESTS,
            body,
            Some("req-1".to_string()),
        );
        match err {
            KalshiError::Http {
                status,
                api_error,
                raw_body,
                request_id,
            } => {
                assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
                assert_eq!(request_id.as_deref(), Some("req-1"));
                assert!(raw_body.contains("rate_limit"));
                let api_error = api_error.expect("expected parsed error body");
                assert_eq!(api_error.code.as_deref(), Some("rate_limit"));
                assert_eq!(api_error.message.as_deref(), Some("too fast"));
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn http_error_handles_non_json_body() {
        let body = b"plain error body";
        let err = build_http_error(StatusCode::BAD_REQUEST, body, None);
        match err {
            KalshiError::Http {
                status,
                api_error,
                raw_body,
                request_id,
            } => {
                assert_eq!(status, StatusCode::BAD_REQUEST);
                assert!(api_error.is_none());
                assert_eq!(raw_body, "plain error body");
                assert!(request_id.is_none());
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn http_error_parses_wrapped_error_envelope() {
        let body = br#"{"error":{"code":"bad_request","message":"invalid"}}"#;
        let err = build_http_error(StatusCode::BAD_REQUEST, body, None);
        match err {
            KalshiError::Http { api_error, .. } => {
                let api_error = api_error.expect("expected api error");
                assert_eq!(api_error.code.as_deref(), Some("bad_request"));
                assert_eq!(api_error.message.as_deref(), Some("invalid"));
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn rest_success_parse_error_exposes_raw_bytes_and_reason() {
        let body = r#"{"exchange_active":"true","trading_active":true}"#;
        let (rest_origin, _hits, server) =
            spawn_http_sequence_server(vec![TestHttpResponse::new(StatusCode::OK, body)]).await;

        let client = KalshiRestClient::builder(test_env(rest_origin))
            .build()
            .expect("build client");

        let err = client
            .get_exchange_status()
            .await
            .expect_err("invalid response schema should fail");
        match err {
            KalshiError::Parse {
                context,
                reason,
                raw,
                ..
            } => {
                assert_eq!(
                    context,
                    format!(
                        "REST GET {}",
                        KalshiRestClient::full_path("/exchange/status")
                    )
                );
                assert_eq!(raw, body.as_bytes());
                assert!(reason.contains("invalid type"));
            }
            other => panic!("unexpected error: {other:?}"),
        }

        server.await.expect("server").expect("server ok");
    }

    #[test]
    fn builder_accepts_proxy_result_input() {
        let client = KalshiRestClient::builder(KalshiEnvironment::demo())
            .with_proxy(reqwest::Proxy::all("http://127.0.0.1:8080"))
            .build();
        assert!(client.is_ok());
    }

    #[test]
    fn builder_rejects_invalid_proxy_result_input() {
        let err = KalshiRestClient::builder(KalshiEnvironment::demo())
            .with_proxy(reqwest::Proxy::all("not a url"))
            .build()
            .expect_err("invalid proxy should fail at build");

        match err {
            KalshiError::InvalidParams(message) => {
                assert!(message.contains("invalid proxy configuration"));
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_retries_on_503_then_succeeds() {
        let (rest_origin, hits, server) = spawn_http_sequence_server(vec![
            TestHttpResponse::new(
                StatusCode::SERVICE_UNAVAILABLE,
                r#"{"code":"unavailable","message":"try again"}"#,
            ),
            TestHttpResponse::new(
                StatusCode::OK,
                r#"{"exchange_active":true,"trading_active":true}"#,
            ),
        ])
        .await;

        let client = KalshiRestClient::builder(test_env(rest_origin))
            .with_retry_config(RetryConfig {
                max_retries: 1,
                base_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(1),
                jitter: 0.0,
                retry_non_idempotent: false,
            })
            .build()
            .expect("build client");

        let response = client
            .get_exchange_status()
            .await
            .expect("request succeeds");
        assert!(response.exchange_active);
        assert_eq!(hits.load(Ordering::Relaxed), 2);
        server.await.expect("server").expect("server ok");
    }

    #[tokio::test]
    async fn post_does_not_retry_by_default() {
        let (rest_origin, hits, server) = spawn_http_sequence_server(vec![TestHttpResponse::new(
            StatusCode::SERVICE_UNAVAILABLE,
            r#"{"code":"unavailable","message":"retry me"}"#,
        )])
        .await;

        let client = KalshiRestClient::builder(test_env(rest_origin))
            .with_retry_config(RetryConfig {
                max_retries: 2,
                base_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(1),
                jitter: 0.0,
                retry_non_idempotent: false,
            })
            .build()
            .expect("build client");

        let path = KalshiRestClient::full_path("/test-post");
        let result = client
            .send::<(), _, serde_json::Value>(
                Method::POST,
                &path,
                Option::<&()>::None,
                Some(&json!({"x": 1})),
                false,
            )
            .await;
        assert!(matches!(result, Err(KalshiError::Http { .. })));
        assert_eq!(hits.load(Ordering::Relaxed), 1);
        server.await.expect("server").expect("server ok");
    }

    #[tokio::test]
    async fn post_retry_opt_in_retries_and_succeeds() {
        let (rest_origin, hits, server) = spawn_http_sequence_server(vec![
            TestHttpResponse::new(
                StatusCode::SERVICE_UNAVAILABLE,
                r#"{"code":"unavailable","message":"retry me"}"#,
            ),
            TestHttpResponse::new(StatusCode::OK, r#"{"ok":true}"#),
        ])
        .await;

        let client = KalshiRestClient::builder(test_env(rest_origin))
            .with_retry_config(RetryConfig {
                max_retries: 1,
                base_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(1),
                jitter: 0.0,
                retry_non_idempotent: true,
            })
            .build()
            .expect("build client");

        let path = KalshiRestClient::full_path("/test-post");
        let value = client
            .send::<(), _, serde_json::Value>(
                Method::POST,
                &path,
                Option::<&()>::None,
                Some(&json!({"x": 1})),
                false,
            )
            .await
            .expect("request succeeds after retry");

        assert_eq!(value["ok"], json!(true));
        assert_eq!(hits.load(Ordering::Relaxed), 2);
        server.await.expect("server").expect("server ok");
    }

    #[tokio::test]
    async fn retry_after_header_is_honored_for_429() {
        let (rest_origin, hits, server) = spawn_http_sequence_server(vec![
            TestHttpResponse::new(
                StatusCode::TOO_MANY_REQUESTS,
                r#"{"code":"too_many_requests","message":"slow down"}"#,
            )
            .with_header("Retry-After", "1"),
            TestHttpResponse::new(
                StatusCode::OK,
                r#"{"exchange_active":true,"trading_active":true}"#,
            ),
        ])
        .await;

        let client = KalshiRestClient::builder(test_env(rest_origin))
            .with_retry_config(RetryConfig {
                max_retries: 1,
                base_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(1),
                jitter: 0.0,
                retry_non_idempotent: false,
            })
            .build()
            .expect("build client");

        let start = Instant::now();
        let _ = client
            .get_exchange_status()
            .await
            .expect("request succeeds");
        assert!(start.elapsed() >= Duration::from_millis(900));
        assert_eq!(hits.load(Ordering::Relaxed), 2);
        server.await.expect("server").expect("server ok");
    }

    #[tokio::test]
    async fn request_id_extraction_supports_both_header_names() {
        let (rest_origin_a, _hits_a, server_a) = spawn_http_sequence_server(vec![
            TestHttpResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                r#"{"code":"internal","message":"boom"}"#,
            )
            .with_header("x-request-id", "req-x"),
        ])
        .await;

        let client_a = KalshiRestClient::builder(test_env(rest_origin_a))
            .with_retry_config(RetryConfig {
                max_retries: 0,
                ..Default::default()
            })
            .build()
            .expect("build client");
        let err_a = client_a
            .get_exchange_status()
            .await
            .expect_err("expected error");
        match err_a {
            KalshiError::Http { request_id, .. } => {
                assert_eq!(request_id.as_deref(), Some("req-x"));
            }
            other => panic!("unexpected error: {:?}", other),
        }
        server_a.await.expect("server").expect("server ok");

        let (rest_origin_b, _hits_b, server_b) = spawn_http_sequence_server(vec![
            TestHttpResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                r#"{"code":"internal","message":"boom"}"#,
            )
            .with_header("request-id", "req-alt"),
        ])
        .await;

        let client_b = KalshiRestClient::builder(test_env(rest_origin_b))
            .with_retry_config(RetryConfig {
                max_retries: 0,
                ..Default::default()
            })
            .build()
            .expect("build client");
        let err_b = client_b
            .get_exchange_status()
            .await
            .expect_err("expected error");
        match err_b {
            KalshiError::Http { request_id, .. } => {
                assert_eq!(request_id.as_deref(), Some("req-alt"));
            }
            other => panic!("unexpected error: {:?}", other),
        }
        server_b.await.expect("server").expect("server ok");
    }

    #[tokio::test]
    async fn rate_limiter_zero_rps_returns_quickly() {
        let limiter = RateLimiter::new(RateLimitConfig {
            read_rps: 0,
            write_rps: 0,
        });

        timeout(Duration::from_millis(10), limiter.wait(RateLimitKind::Read))
            .await
            .expect("read wait timed out");
        timeout(
            Duration::from_millis(10),
            limiter.wait(RateLimitKind::Write),
        )
        .await
        .expect("write wait timed out");
    }

    #[tokio::test]
    async fn paginate_cursor_collects_all_pages() {
        let client = KalshiRestClient::new(KalshiEnvironment::demo());
        let mut calls = 0usize;

        let items = client
            .paginate_cursor(Some("c1".to_string()), |cursor| {
                let expected = if calls == 0 {
                    Some("c1".to_string())
                } else {
                    Some("c2".to_string())
                };
                let page = if calls == 0 {
                    (vec![1, 2], Some("c2".to_string()))
                } else {
                    (vec![3], None)
                };
                calls += 1;
                async move {
                    assert_eq!(cursor, expected);
                    Ok(page)
                }
            })
            .await
            .expect("paginate failed");

        assert_eq!(items, vec![1, 2, 3]);
        assert_eq!(calls, 2);
    }

    #[tokio::test]
    async fn cursor_pager_returns_pages_in_order() {
        let mut pages = VecDeque::from(vec![(vec![1, 2], Some("c1".to_string())), (vec![3], None)]);
        let mut pager = CursorPager::new(None, move |_cursor| {
            let page = pages.pop_front().unwrap_or((Vec::<i32>::new(), None));
            Box::pin(async move { Ok(page) })
        });

        let first = pager.next_page().await.unwrap().unwrap();
        assert_eq!(first, vec![1, 2]);
        let second = pager.next_page().await.unwrap().unwrap();
        assert_eq!(second, vec![3]);
        let done = pager.next_page().await.unwrap();
        assert!(done.is_none());
    }

    #[tokio::test]
    async fn stream_items_truncates_without_extra_fetch() {
        let mut pages = VecDeque::from(vec![
            (vec![1, 2], Some("c1".to_string())),
            (vec![3, 4], Some("c2".to_string())),
            (vec![5], None),
        ]);
        let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let call_count_ref = Arc::clone(&call_count);
        let pager = CursorPager::new(None, move |_cursor| {
            call_count_ref.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let page = pages.pop_front().unwrap_or((Vec::<i32>::new(), None));
            Box::pin(async move { Ok(page) })
        });

        let items: Vec<i32> = stream_items(pager, Some(3)).try_collect().await.unwrap();

        assert_eq!(items, vec![1, 2, 3]);
        // Should only fetch as many pages as needed to reach 3 items.
        assert_eq!(call_count.load(std::sync::atomic::Ordering::Relaxed), 2);
    }
}
