use crate::rest::types::*;
use crate::types::ErrorResponse;
use crate::{KalshiAuth, KalshiEnvironment, KalshiError, REST_PREFIX};

use futures::future::BoxFuture;
use futures::stream::{self, Stream};
use rand::random;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, RETRY_AFTER};
use reqwest::{Client, Method, Proxy, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, sleep};
use url::Url;

/// Per-second rate limits for read (GET) and write (POST/DELETE) requests.
///
/// The Kalshi API enforces separate rate limits for reads and writes.
/// Set either value to `0` to disable throttling for that category.
///
/// # Default
///
/// The default matches the **Basic** tier: 20 read RPS, 10 write RPS.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    /// Maximum GET requests per second (0 = unlimited).
    pub read_rps: u32,
    /// Maximum POST/DELETE requests per second (0 = unlimited).
    pub write_rps: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        // Basic tier defaults.
        Self {
            read_rps: 20,
            write_rps: 10,
        }
    }
}

/// Named rate-limit tier matching Kalshi's published API tiers.
///
/// Pass to [`KalshiRestClient::with_rate_limit_tier`] for quick configuration.
#[derive(Debug, Clone, Copy)]
pub enum RateLimitTier {
    /// 20 read RPS, 10 write RPS.
    Basic,
}

impl RateLimitTier {
    fn config(self) -> RateLimitConfig {
        match self {
            RateLimitTier::Basic => RateLimitConfig::default(),
        }
    }
}

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
    fn allows_method(&self, method: &Method) -> bool {
        matches!(*method, Method::GET | Method::DELETE)
            || (self.retry_non_idempotent
                && matches!(*method, Method::POST | Method::PUT | Method::PATCH))
    }

    fn backoff_delay(&self, retry_number: u32) -> Duration {
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

#[derive(Debug, Clone, Copy)]
enum RateLimitKind {
    Read,
    Write,
}

fn rate_limit_kind(method: &Method) -> RateLimitKind {
    if *method == Method::GET {
        RateLimitKind::Read
    } else {
        RateLimitKind::Write
    }
}

fn build_http_error(
    status: reqwest::StatusCode,
    bytes: &[u8],
    request_id: Option<String>,
) -> KalshiError {
    #[derive(serde::Deserialize)]
    struct WrappedErrorBody {
        error: ErrorResponse,
    }

    let raw_body = String::from_utf8_lossy(bytes).to_string();
    let normalize = |error: ErrorResponse| {
        if error.code.is_some()
            || error.message.is_some()
            || error.details.is_some()
            || error.service.is_some()
        {
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

fn retryable_status(status: StatusCode) -> bool {
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

fn retryable_reqwest_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect() || err.is_request()
}

fn retry_after_delay(headers: &HeaderMap) -> Option<Duration> {
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

#[derive(Debug)]
struct RateLimiter {
    read: Mutex<Instant>,
    write: Mutex<Instant>,
    read_interval: Duration,
    write_interval: Duration,
}

impl RateLimiter {
    fn new(config: RateLimitConfig) -> Self {
        let read_interval = if config.read_rps == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_secs_f64(1.0 / config.read_rps as f64)
        };
        let write_interval = if config.write_rps == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_secs_f64(1.0 / config.write_rps as f64)
        };

        let now = Instant::now();
        Self {
            read: Mutex::new(now - read_interval),
            write: Mutex::new(now - write_interval),
            read_interval,
            write_interval,
        }
    }

    async fn wait(&self, kind: RateLimitKind) {
        let (lock, interval) = match kind {
            RateLimitKind::Read => (&self.read, self.read_interval),
            RateLimitKind::Write => (&self.write, self.write_interval),
        };

        if interval.is_zero() {
            return;
        }

        let mut last = lock.lock().await;
        let now = Instant::now();
        let scheduled = if *last + interval > now {
            *last + interval
        } else {
            now
        };
        *last = scheduled;
        drop(last);

        if scheduled > now {
            tokio::time::sleep(scheduled - now).await;
        }
    }
}

/// Manual page-by-page cursor pagination.
///
/// Use `CursorPager` when you need:
/// - Explicit control over when to fetch the next page
/// - Access to page boundaries (e.g., for batch processing)
/// - Custom termination logic based on page contents
///
/// For item-by-item iteration, see the `stream_*` methods on [`KalshiRestClient`].
///
/// # Example
/// ```no_run
/// # use kalshi_fast::{KalshiEnvironment, KalshiRestClient, GetMarketsParams};
/// # async fn example() -> Result<(), kalshi_fast::KalshiError> {
/// let client = KalshiRestClient::new(KalshiEnvironment::demo());
/// let mut pager = client.markets_pager(GetMarketsParams::default());
///
/// while let Some(markets) = pager.next_page().await? {
///     println!("Got {} markets", markets.len());
/// }
/// # Ok(())
/// # }
/// ```
pub struct CursorPager<T> {
    cursor: Option<String>,
    done: bool,
    fetch: Box<
        dyn FnMut(
                Option<String>,
            ) -> BoxFuture<'static, Result<(Vec<T>, Option<String>), KalshiError>>
            + Send,
    >,
}

impl<T> CursorPager<T> {
    pub fn new<F>(cursor: Option<String>, fetch: F) -> Self
    where
        F: FnMut(
                Option<String>,
            ) -> BoxFuture<'static, Result<(Vec<T>, Option<String>), KalshiError>>
            + Send
            + 'static,
    {
        Self {
            cursor: cursor.filter(|c| !c.is_empty()),
            done: false,
            fetch: Box::new(fetch),
        }
    }

    /// Fetch the next page of results.
    ///
    /// Returns `Ok(Some(items))` if there are more results, `Ok(None)` when
    /// pagination is complete, or `Err` on failure.
    pub async fn next_page(&mut self) -> Result<Option<Vec<T>>, KalshiError> {
        if self.done {
            return Ok(None);
        }

        let (items, next) = (self.fetch)(self.cursor.clone()).await?;
        self.cursor = next.filter(|c| !c.is_empty());
        if self.cursor.is_none() {
            self.done = true;
        }

        Ok(Some(items))
    }

    /// Returns the cursor for the next page fetch.
    ///
    /// Useful for checkpointing/resuming pagination across sessions.
    pub fn current_cursor(&self) -> Option<&str> {
        self.cursor.as_deref()
    }

    /// Returns true if pagination is complete.
    pub fn is_done(&self) -> bool {
        self.done
    }
}

struct StreamState<T> {
    pager: CursorPager<T>,
    buffer: VecDeque<T>,
    remaining: Option<usize>,
    done: bool,
}

/// Stream items one-by-one from paginated endpoints.
///
/// Streams provide lazy, item-level iteration built on [`CursorPager`].
/// Pages are fetched on-demand; use `max_items` for early termination.
///
/// # Pagers vs Streams
///
/// | Aspect | Pager | Stream |
/// |--------|-------|--------|
/// | Returns | Full pages (`Vec<T>`) | Individual items |
/// | Control | Manual `next_page()` | Async iterator |
/// | Early stop | Stop calling `next_page()` | `max_items` or `.take()` |
/// | Use case | Batch processing, checkpointing | Item processing, collecting subsets |
fn stream_items<T>(
    pager: CursorPager<T>,
    max_items: Option<usize>,
) -> impl Stream<Item = Result<T, KalshiError>> + Send
where
    T: Send + 'static,
{
    let state = StreamState {
        pager,
        buffer: VecDeque::new(),
        remaining: max_items,
        done: false,
    };

    stream::unfold(state, |mut state| async move {
        if state.done {
            return None;
        }
        if let Some(remaining) = state.remaining
            && remaining == 0
        {
            return None;
        }

        loop {
            if let Some(item) = state.buffer.pop_front() {
                if let Some(remaining) = state.remaining.as_mut() {
                    *remaining -= 1;
                }
                return Some((Ok(item), state));
            }

            match state.pager.next_page().await {
                Ok(Some(items)) => {
                    state.buffer = items.into();
                    if state.buffer.is_empty() && state.pager.done {
                        return None;
                    }
                }
                Ok(None) => {
                    return None;
                }
                Err(err) => {
                    state.done = true;
                    return Some((Err(err), state));
                }
            }
        }
    })
}

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

    fn full_path(endpoint_path: &str) -> String {
        // endpoint_path must begin with "/", e.g. "/markets"
        format!("{REST_PREFIX}{endpoint_path}")
    }

    fn build_url(&self, full_path: &str) -> Result<Url, KalshiError> {
        Ok(self.rest_origin.join(full_path)?)
    }

    fn insert_auth_headers(
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

    async fn send<Q, B, T>(
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

    // -----------------------------------------------
    // Series
    // -----------------------------------------------

    /// List all series, optionally filtered by category or tags.
    pub async fn get_series_list(
        &self,
        params: GetSeriesListParams,
    ) -> Result<GetSeriesListResponse, KalshiError> {
        let path = Self::full_path("/series");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get a single series by ticker.
    pub async fn get_series(&self, series_ticker: &str) -> Result<GetSeriesResponse, KalshiError> {
        let path = Self::full_path(&format!("/series/{series_ticker}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    // -----------------------------------------------
    // Events
    // -----------------------------------------------

    /// List events (excludes multivariate events). Supports cursor pagination.
    pub async fn get_events(
        &self,
        params: GetEventsParams,
    ) -> Result<GetEventsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/events");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get a single event by ticker, optionally including its nested markets.
    pub async fn get_event(
        &self,
        event_ticker: &str,
        with_nested_markets: Option<bool>,
    ) -> Result<GetEventResponse, KalshiError> {
        let path = Self::full_path(&format!("/events/{event_ticker}"));
        let params = GetEventParams {
            with_nested_markets,
        };
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    // -----------------------------------------------
    // Markets
    // -----------------------------------------------

    /// List markets with optional filters. Supports cursor pagination.
    pub async fn get_markets(
        &self,
        params: GetMarketsParams,
    ) -> Result<GetMarketsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/markets");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get a single market by ticker.
    pub async fn get_market(&self, market_ticker: &str) -> Result<GetMarketResponse, KalshiError> {
        let path = Self::full_path(&format!("/markets/{market_ticker}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get a single historical market by ticker.
    pub async fn get_historical_market(
        &self,
        market_ticker: &str,
    ) -> Result<GetMarketResponse, KalshiError> {
        let path = Self::full_path(&format!("/historical/markets/{market_ticker}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get the order book for a market, optionally limited to `depth` levels per side.
    pub async fn get_market_orderbook(
        &self,
        market_ticker: &str,
        depth: Option<u32>,
    ) -> Result<GetMarketOrderbookResponse, KalshiError> {
        let path = Self::full_path(&format!("/markets/{market_ticker}/orderbook"));
        let params = GetMarketOrderbookParams { depth };
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    // -----------------------------------------------
    // Trades
    // -----------------------------------------------

    /// List public trades. Supports cursor pagination.
    pub async fn get_trades(
        &self,
        params: GetTradesParams,
    ) -> Result<GetTradesResponse, KalshiError> {
        let path = Self::full_path("/markets/trades");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// List historical fills. Requires auth.
    pub async fn get_historical_fills(
        &self,
        params: GetHistoricalFillsParams,
    ) -> Result<GetFillsResponse, KalshiError> {
        let path = Self::full_path("/historical/fills");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// List historical orders. Requires auth.
    pub async fn get_historical_orders(
        &self,
        params: GetHistoricalOrdersParams,
    ) -> Result<GetOrdersResponse, KalshiError> {
        let path = Self::full_path("/historical/orders");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// List historical markets.
    pub async fn get_historical_markets(
        &self,
        params: GetHistoricalMarketsParams,
    ) -> Result<GetMarketsResponse, KalshiError> {
        let path = Self::full_path("/historical/markets");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get historical data cutoffs that separate live and historical datasets.
    pub async fn get_historical_cutoff(&self) -> Result<GetHistoricalCutoffResponse, KalshiError> {
        let path = Self::full_path("/historical/cutoff");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    // -----------------------------------------------
    // Exchange
    // -----------------------------------------------

    /// Get the current exchange status (open, closed, etc.).
    pub async fn get_exchange_status(&self) -> Result<GetExchangeStatusResponse, KalshiError> {
        let path = Self::full_path("/exchange/status");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get exchange announcements.
    pub async fn get_exchange_announcements(
        &self,
    ) -> Result<GetExchangeAnnouncementsResponse, KalshiError> {
        let path = Self::full_path("/exchange/announcements");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get the exchange trading schedule.
    pub async fn get_exchange_schedule(&self) -> Result<GetExchangeScheduleResponse, KalshiError> {
        let path = Self::full_path("/exchange/schedule");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get the timestamp of the latest user-data change (useful for cache invalidation).
    pub async fn get_user_data_timestamp(
        &self,
    ) -> Result<GetUserDataTimestampResponse, KalshiError> {
        let path = Self::full_path("/exchange/user_data_timestamp");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// List fee changes for a series.
    pub async fn get_series_fee_changes(
        &self,
        params: GetSeriesFeeChangesParams,
    ) -> Result<GetSeriesFeeChangesResponse, KalshiError> {
        let path = Self::full_path("/series/fee_changes");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    // -----------------------------------------------
    // Portfolio (authenticated)
    // -----------------------------------------------

    /// Get the account balance.
    ///
    /// **Requires auth.**
    pub async fn get_balance(&self) -> Result<GetBalanceResponse, KalshiError> {
        let path = Self::full_path("/portfolio/balance");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// List open positions. Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_positions(
        &self,
        params: GetPositionsParams,
    ) -> Result<GetPositionsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/portfolio/positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// List orders with optional filters. Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_orders(
        &self,
        params: GetOrdersParams,
    ) -> Result<GetOrdersResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/portfolio/orders");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    // -----------------------------------------------
    // Orders (authenticated)
    // -----------------------------------------------

    /// Place a new order.
    ///
    /// **Requires auth.**
    pub async fn create_order(
        &self,
        body: CreateOrderRequest,
    ) -> Result<CreateOrderResponse, KalshiError> {
        let path = Self::full_path("/portfolio/orders");
        body.validate()?;
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// Cancel an order by ID.
    ///
    /// **Requires auth.**
    pub async fn cancel_order(
        &self,
        order_id: &str,
        params: CancelOrderParams,
    ) -> Result<CancelOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}"));
        self.send(
            Method::DELETE,
            &path,
            Some(&params),
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// List fills (executed trades). Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_fills(&self, params: GetFillsParams) -> Result<GetFillsResponse, KalshiError> {
        let path = Self::full_path("/portfolio/fills");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// List settlements. Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_settlements(
        &self,
        params: GetSettlementsParams,
    ) -> Result<GetSettlementsResponse, KalshiError> {
        let path = Self::full_path("/portfolio/settlements");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    // -----------------------------------------------
    // Account (authenticated)
    // -----------------------------------------------

    /// Get API rate-limit and position limits for the account.
    ///
    /// **Requires auth.**
    pub async fn get_account_api_limits(&self) -> Result<GetAccountApiLimitsResponse, KalshiError> {
        let path = Self::full_path("/account/limits");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    // -----------------------------------------------
    // Subaccounts (authenticated)
    // -----------------------------------------------

    /// Create a new subaccount.
    ///
    /// **Requires auth.**
    pub async fn create_subaccount(&self) -> Result<CreateSubaccountResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts");
        self.send(
            Method::POST,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// Get balances for all subaccounts.
    ///
    /// **Requires auth.**
    pub async fn get_subaccount_balances(
        &self,
    ) -> Result<GetSubaccountBalancesResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts/balances");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// Transfer funds between subaccounts.
    ///
    /// **Requires auth.**
    pub async fn transfer_subaccount(
        &self,
        body: ApplySubaccountTransferRequest,
    ) -> Result<ApplySubaccountTransferResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts/transfer");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// List subaccount transfers. Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_subaccount_transfers(
        &self,
        params: GetSubaccountTransfersParams,
    ) -> Result<GetSubaccountTransfersResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts/transfers");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// Get subaccount netting configuration.
    ///
    /// **Requires auth.**
    pub async fn get_subaccount_netting(
        &self,
    ) -> Result<GetSubaccountNettingResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts/netting");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// Update netting configuration for a subaccount.
    ///
    /// **Requires auth.**
    pub async fn update_subaccount_netting(
        &self,
        body: UpdateSubaccountNettingRequest,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts/netting");
        self.send(Method::PUT, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    // -----------------------------------------------
    // API keys (authenticated)
    // -----------------------------------------------

    pub async fn get_api_keys(&self) -> Result<GetApiKeysResponse, KalshiError> {
        let path = Self::full_path("/api_keys");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn create_api_key(
        &self,
        body: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse, KalshiError> {
        let path = Self::full_path("/api_keys");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn generate_api_key(
        &self,
        body: GenerateApiKeyRequest,
    ) -> Result<GenerateApiKeyResponse, KalshiError> {
        let path = Self::full_path("/api_keys/generate");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn delete_api_key(&self, api_key: &str) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/api_keys/{api_key}"));
        self.send(
            Method::DELETE,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    // -----------------------------------------------
    // Communications (authenticated)
    // -----------------------------------------------

    pub async fn get_communications_id(&self) -> Result<GetCommunicationsIdResponse, KalshiError> {
        let path = Self::full_path("/communications/id");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn get_rfqs(&self, params: GetRFQsParams) -> Result<GetRFQsResponse, KalshiError> {
        let path = Self::full_path("/communications/rfqs");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn create_rfq(
        &self,
        body: CreateRFQRequest,
    ) -> Result<CreateRFQResponse, KalshiError> {
        let path = Self::full_path("/communications/rfqs");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_rfq(&self, rfq_id: &str) -> Result<GetRFQResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/rfqs/{rfq_id}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn delete_rfq(&self, rfq_id: &str) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/rfqs/{rfq_id}"));
        self.send(
            Method::DELETE,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn get_quotes(
        &self,
        params: GetQuotesParams,
    ) -> Result<GetQuotesResponse, KalshiError> {
        let path = Self::full_path("/communications/quotes");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn create_quote(
        &self,
        body: CreateQuoteRequest,
    ) -> Result<CreateQuoteResponse, KalshiError> {
        let path = Self::full_path("/communications/quotes");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_quote(&self, quote_id: &str) -> Result<GetQuoteResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/quotes/{quote_id}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn delete_quote(&self, quote_id: &str) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/quotes/{quote_id}"));
        self.send(
            Method::DELETE,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn accept_quote(
        &self,
        quote_id: &str,
        body: AcceptQuoteRequest,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/quotes/{quote_id}/accept"));
        self.send(Method::PUT, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn confirm_quote(&self, quote_id: &str) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/quotes/{quote_id}/confirm"));
        let body = EmptyResponse::default();
        self.send(Method::PUT, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    // -----------------------------------------------
    // Additional public endpoints
    // -----------------------------------------------

    pub async fn get_multivariate_events(
        &self,
        params: GetMultivariateEventsParams,
    ) -> Result<GetMultivariateEventsResponse, KalshiError> {
        let path = Self::full_path("/events/multivariate");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_event_metadata(
        &self,
        event_ticker: &str,
    ) -> Result<GetEventMetadataResponse, KalshiError> {
        let path = Self::full_path(&format!("/events/{event_ticker}/metadata"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_incentive_programs(
        &self,
        params: GetIncentiveProgramsParams,
    ) -> Result<GetIncentiveProgramsResponse, KalshiError> {
        let path = Self::full_path("/incentive_programs");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_live_data_batch(
        &self,
        params: GetLiveDatasParams,
    ) -> Result<GetLiveDatasResponse, KalshiError> {
        let path = Self::full_path("/live_data/batch");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_live_data(
        &self,
        live_data_type: &str,
        milestone_id: &str,
    ) -> Result<GetLiveDataResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/live_data/{live_data_type}/milestone/{milestone_id}"
        ));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn batch_get_market_candlesticks(
        &self,
        params: BatchGetMarketCandlesticksParams,
    ) -> Result<BatchGetMarketCandlesticksResponse, KalshiError> {
        let path = Self::full_path("/markets/candlesticks");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_milestones(
        &self,
        params: GetMilestonesParams,
    ) -> Result<GetMilestonesResponse, KalshiError> {
        let path = Self::full_path("/milestones");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_milestone(
        &self,
        milestone_id: &str,
    ) -> Result<GetMilestoneResponse, KalshiError> {
        let path = Self::full_path(&format!("/milestones/{milestone_id}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_multivariate_event_collections(
        &self,
        params: GetMultivariateEventCollectionsParams,
    ) -> Result<GetMultivariateEventCollectionsResponse, KalshiError> {
        let path = Self::full_path("/multivariate_event_collections");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_multivariate_event_collection(
        &self,
        collection_ticker: &str,
    ) -> Result<GetMultivariateEventCollectionResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/multivariate_event_collections/{collection_ticker}"
        ));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn create_market_in_multivariate_event_collection(
        &self,
        collection_ticker: &str,
        body: CreateMarketInMultivariateEventCollectionRequest,
    ) -> Result<CreateMarketInMultivariateEventCollectionResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/multivariate_event_collections/{collection_ticker}"
        ));
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_multivariate_event_collection_lookup_history(
        &self,
        collection_ticker: &str,
        params: GetMultivariateEventCollectionLookupHistoryParams,
    ) -> Result<GetMultivariateEventCollectionLookupHistoryResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/multivariate_event_collections/{collection_ticker}/lookup"
        ));
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn lookup_tickers_for_market_in_multivariate_event_collection(
        &self,
        collection_ticker: &str,
        body: LookupTickersForMarketInMultivariateEventCollectionRequest,
    ) -> Result<LookupTickersForMarketInMultivariateEventCollectionResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/multivariate_event_collections/{collection_ticker}/lookup"
        ));
        self.send(Method::PUT, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_historical_market_candlesticks(
        &self,
        ticker: &str,
        params: GetMarketCandlesticksHistoricalParams,
    ) -> Result<GetMarketCandlesticksHistoricalResponse, KalshiError> {
        let path = Self::full_path(&format!("/historical/markets/{ticker}/candlesticks"));
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_market_candlesticks(
        &self,
        series_ticker: &str,
        ticker: &str,
        params: GetMarketCandlesticksParams,
    ) -> Result<GetMarketCandlesticksResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/series/{series_ticker}/markets/{ticker}/candlesticks"
        ));
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_event_market_candlesticks(
        &self,
        series_ticker: &str,
        ticker: &str,
        params: GetEventCandlesticksParams,
    ) -> Result<GetEventCandlesticksResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/series/{series_ticker}/events/{ticker}/candlesticks"
        ));
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_event_forecast_percentile_history(
        &self,
        series_ticker: &str,
        ticker: &str,
        params: GetEventForecastPercentileHistoryParams,
    ) -> Result<GetEventForecastPercentilesHistoryResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/series/{series_ticker}/events/{ticker}/forecast_percentile_history"
        ));
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn get_structured_targets(
        &self,
        params: GetStructuredTargetsParams,
    ) -> Result<GetStructuredTargetsResponse, KalshiError> {
        let path = Self::full_path("/structured_targets");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_structured_target(
        &self,
        structured_target_id: &str,
    ) -> Result<GetStructuredTargetResponse, KalshiError> {
        let path = Self::full_path(&format!("/structured_targets/{structured_target_id}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    // -----------------------------------------------
    // Additional authenticated endpoints
    // -----------------------------------------------

    pub async fn get_fcm_orders(
        &self,
        params: GetFcmOrdersParams,
    ) -> Result<GetFcmOrdersResponse, KalshiError> {
        let path = Self::full_path("/fcm/orders");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn get_fcm_positions(
        &self,
        params: GetFcmPositionsParams,
    ) -> Result<GetFcmPositionsResponse, KalshiError> {
        let path = Self::full_path("/fcm/positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn get_order_groups(
        &self,
        params: SubaccountQueryParams,
    ) -> Result<GetOrderGroupsResponse, KalshiError> {
        let path = Self::full_path("/portfolio/order_groups");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn create_order_group(
        &self,
        body: CreateOrderGroupRequest,
    ) -> Result<CreateOrderGroupResponse, KalshiError> {
        let path = Self::full_path("/portfolio/order_groups/create");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_order_group(
        &self,
        order_group_id: &str,
        params: SubaccountQueryParams,
    ) -> Result<GetOrderGroupResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}"));
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn delete_order_group(
        &self,
        order_group_id: &str,
        params: SubaccountQueryParams,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}"));
        self.send(
            Method::DELETE,
            &path,
            Some(&params),
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn update_order_group_limit(
        &self,
        order_group_id: &str,
        body: UpdateOrderGroupLimitRequest,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}/limit"));
        self.send(Method::PUT, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn reset_order_group(
        &self,
        order_group_id: &str,
        params: SubaccountQueryParams,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}/reset"));
        let body = EmptyResponse::default();
        self.send(Method::PUT, &path, Some(&params), Some(&body), true)
            .await
    }

    pub async fn trigger_order_group(
        &self,
        order_group_id: &str,
        params: SubaccountQueryParams,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}/trigger"));
        let body = EmptyResponse::default();
        self.send(Method::PUT, &path, Some(&params), Some(&body), true)
            .await
    }

    pub async fn batch_create_orders(
        &self,
        body: BatchCreateOrdersRequest,
    ) -> Result<BatchCreateOrdersResponse, KalshiError> {
        let path = Self::full_path("/portfolio/orders/batched");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn batch_cancel_orders(
        &self,
        body: BatchCancelOrdersRequest,
    ) -> Result<BatchCancelOrdersResponse, KalshiError> {
        let path = Self::full_path("/portfolio/orders/batched");
        self.send(
            Method::DELETE,
            &path,
            Option::<&()>::None,
            Some(&body),
            true,
        )
        .await
    }

    pub async fn get_order(&self, order_id: &str) -> Result<GetOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn amend_order(
        &self,
        order_id: &str,
        body: AmendOrderRequest,
    ) -> Result<AmendOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}/amend"));
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn decrease_order(
        &self,
        order_id: &str,
        body: DecreaseOrderRequest,
    ) -> Result<DecreaseOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}/decrease"));
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_order_queue_positions(
        &self,
        params: GetOrderQueuePositionsParams,
    ) -> Result<GetOrderQueuePositionsResponse, KalshiError> {
        let path = Self::full_path("/portfolio/orders/queue_positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn get_order_queue_position(
        &self,
        order_id: &str,
    ) -> Result<GetOrderQueuePositionResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}/queue_position"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn get_portfolio_total_resting_order_value(
        &self,
    ) -> Result<GetPortfolioRestingOrderTotalValueResponse, KalshiError> {
        let path = Self::full_path("/portfolio/summary/total_resting_order_value");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn get_tags_by_categories(
        &self,
    ) -> Result<GetTagsForSeriesCategoriesResponse, KalshiError> {
        let path = Self::full_path("/search/tags_by_categories");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_filters_by_sport(&self) -> Result<GetFiltersBySportsResponse, KalshiError> {
        let path = Self::full_path("/search/filters_by_sport");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

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

    // -----------------------------------------------
    // Pagers — page-level cursor iteration
    // -----------------------------------------------

    /// Create a pager for iterating over events page by page.
    ///
    /// # Example
    /// ```no_run
    /// # use kalshi_fast::{KalshiEnvironment, KalshiRestClient, GetEventsParams};
    /// # async fn example() -> Result<(), kalshi_fast::KalshiError> {
    /// let client = KalshiRestClient::new(KalshiEnvironment::demo());
    /// let mut pager = client.events_pager(GetEventsParams::default());
    ///
    /// while let Some(events) = pager.next_page().await? {
    ///     for event in events {
    ///         println!("{}", event.event_ticker);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn events_pager(&self, params: GetEventsParams) -> CursorPager<EventData> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_events(page_params).await?;
                Ok((resp.events, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over markets page by page. See [`CursorPager`].
    pub fn markets_pager(&self, params: GetMarketsParams) -> CursorPager<Market> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_markets(page_params).await?;
                Ok((resp.markets, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over trades page by page. See [`CursorPager`].
    pub fn trades_pager(&self, params: GetTradesParams) -> CursorPager<Trade> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_trades(page_params).await?;
                Ok((resp.trades, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over positions page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn positions_pager(&self, params: GetPositionsParams) -> CursorPager<PositionsPage> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_positions(page_params).await?;
                let cursor = resp.cursor.clone();
                let page = PositionsPage::from(resp);
                Ok((vec![page], cursor))
            })
        })
    }

    /// Create a pager for iterating over orders page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn orders_pager(&self, params: GetOrdersParams) -> CursorPager<Order> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_orders(page_params).await?;
                Ok((resp.orders, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over fills page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn fills_pager(&self, params: GetFillsParams) -> CursorPager<Fill> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_fills(page_params).await?;
                Ok((resp.fills, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over settlements page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn settlements_pager(&self, params: GetSettlementsParams) -> CursorPager<Settlement> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_settlements(page_params).await?;
                Ok((resp.settlements, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over subaccount transfers page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn subaccount_transfers_pager(
        &self,
        params: GetSubaccountTransfersParams,
    ) -> CursorPager<SubaccountTransfer> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_subaccount_transfers(page_params).await?;
                Ok((resp.subaccount_transfers, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over milestones page by page.
    pub fn milestones_pager(&self, params: GetMilestonesParams) -> CursorPager<Milestone> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_milestones(page_params).await?;
                Ok((resp.milestones, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over multivariate events page by page.
    pub fn multivariate_events_pager(
        &self,
        params: GetMultivariateEventsParams,
    ) -> CursorPager<EventData> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_multivariate_events(page_params).await?;
                Ok((resp.events, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over multivariate event collections page by page.
    pub fn multivariate_event_collections_pager(
        &self,
        params: GetMultivariateEventCollectionsParams,
    ) -> CursorPager<MultivariateEventCollection> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client
                    .get_multivariate_event_collections(page_params)
                    .await?;
                Ok((resp.multivariate_contracts, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over RFQs page by page.
    pub fn rfqs_pager(&self, params: GetRFQsParams) -> CursorPager<RFQ> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_rfqs(page_params).await?;
                Ok((resp.rfqs, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over quotes page by page.
    pub fn quotes_pager(&self, params: GetQuotesParams) -> CursorPager<Quote> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_quotes(page_params).await?;
                Ok((resp.quotes, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over structured targets page by page.
    pub fn structured_targets_pager(
        &self,
        params: GetStructuredTargetsParams,
    ) -> CursorPager<StructuredTarget> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_structured_targets(page_params).await?;
                Ok((resp.structured_targets, resp.cursor))
            })
        })
    }

    // -----------------------------------------------
    // Streams — item-level async iteration
    // -----------------------------------------------

    /// Stream events one by one.
    ///
    /// # Example
    /// ```no_run
    /// # use kalshi_fast::{KalshiEnvironment, KalshiRestClient, GetEventsParams};
    /// # use futures::stream::TryStreamExt;
    /// # async fn example() -> Result<(), kalshi_fast::KalshiError> {
    /// let client = KalshiRestClient::new(KalshiEnvironment::demo());
    /// let events: Vec<_> = client
    ///     .stream_events(GetEventsParams::default(), Some(10))
    ///     .try_collect()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream_events(
        &self,
        params: GetEventsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<EventData, KalshiError>> + Send {
        stream_items(self.events_pager(params), max_items)
    }

    /// Stream markets one by one.
    pub fn stream_markets(
        &self,
        params: GetMarketsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Market, KalshiError>> + Send {
        stream_items(self.markets_pager(params), max_items)
    }

    /// Stream trades one by one.
    pub fn stream_trades(
        &self,
        params: GetTradesParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Trade, KalshiError>> + Send {
        stream_items(self.trades_pager(params), max_items)
    }

    /// Stream positions one by one.
    ///
    /// **Requires auth.**
    pub fn stream_positions(
        &self,
        params: GetPositionsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<PositionsPage, KalshiError>> + Send {
        stream_items(self.positions_pager(params), max_items)
    }

    /// Stream orders one by one.
    ///
    /// **Requires auth.**
    pub fn stream_orders(
        &self,
        params: GetOrdersParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Order, KalshiError>> + Send {
        stream_items(self.orders_pager(params), max_items)
    }

    /// Stream fills one by one.
    ///
    /// **Requires auth.**
    pub fn stream_fills(
        &self,
        params: GetFillsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Fill, KalshiError>> + Send {
        stream_items(self.fills_pager(params), max_items)
    }

    /// Stream settlements one by one.
    ///
    /// **Requires auth.**
    pub fn stream_settlements(
        &self,
        params: GetSettlementsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Settlement, KalshiError>> + Send {
        stream_items(self.settlements_pager(params), max_items)
    }

    /// Stream subaccount transfers one by one.
    ///
    /// **Requires auth.**
    pub fn stream_subaccount_transfers(
        &self,
        params: GetSubaccountTransfersParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<SubaccountTransfer, KalshiError>> + Send {
        stream_items(self.subaccount_transfers_pager(params), max_items)
    }

    /// Stream milestones one by one.
    pub fn stream_milestones(
        &self,
        params: GetMilestonesParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Milestone, KalshiError>> + Send {
        stream_items(self.milestones_pager(params), max_items)
    }

    /// Stream multivariate events one by one.
    pub fn stream_multivariate_events(
        &self,
        params: GetMultivariateEventsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<EventData, KalshiError>> + Send {
        stream_items(self.multivariate_events_pager(params), max_items)
    }

    /// Stream multivariate event collections one by one.
    pub fn stream_multivariate_event_collections(
        &self,
        params: GetMultivariateEventCollectionsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<MultivariateEventCollection, KalshiError>> + Send {
        stream_items(self.multivariate_event_collections_pager(params), max_items)
    }

    /// Stream RFQs one by one.
    pub fn stream_rfqs(
        &self,
        params: GetRFQsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<RFQ, KalshiError>> + Send {
        stream_items(self.rfqs_pager(params), max_items)
    }

    /// Stream quotes one by one.
    pub fn stream_quotes(
        &self,
        params: GetQuotesParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Quote, KalshiError>> + Send {
        stream_items(self.quotes_pager(params), max_items)
    }

    /// Stream structured targets one by one.
    pub fn stream_structured_targets(
        &self,
        params: GetStructuredTargetsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<StructuredTarget, KalshiError>> + Send {
        stream_items(self.structured_targets_pager(params), max_items)
    }

    // -----------------------------------------------
    // Collect-all convenience methods
    // -----------------------------------------------

    /// Fetch all pages for markets using cursor pagination.
    pub async fn get_markets_all(
        &self,
        params: GetMarketsParams,
    ) -> Result<Vec<Market>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_markets(page_params).await?;
                Ok((resp.markets, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for events using cursor pagination.
    pub async fn get_events_all(
        &self,
        params: GetEventsParams,
    ) -> Result<Vec<EventData>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_events(page_params).await?;
                Ok((resp.events, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for trades using cursor pagination.
    pub async fn get_trades_all(&self, params: GetTradesParams) -> Result<Vec<Trade>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_trades(page_params).await?;
                Ok((resp.trades, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for subaccount transfers using cursor pagination.
    pub async fn get_subaccount_transfers_all(
        &self,
        params: GetSubaccountTransfersParams,
    ) -> Result<Vec<SubaccountTransfer>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_subaccount_transfers(page_params).await?;
                Ok((resp.subaccount_transfers, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for milestones using cursor pagination.
    pub async fn get_milestones_all(
        &self,
        params: GetMilestonesParams,
    ) -> Result<Vec<Milestone>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_milestones(page_params).await?;
                Ok((resp.milestones, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for multivariate events using cursor pagination.
    pub async fn get_multivariate_events_all(
        &self,
        params: GetMultivariateEventsParams,
    ) -> Result<Vec<EventData>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_multivariate_events(page_params).await?;
                Ok((resp.events, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for multivariate collections using cursor pagination.
    pub async fn get_multivariate_event_collections_all(
        &self,
        params: GetMultivariateEventCollectionsParams,
    ) -> Result<Vec<MultivariateEventCollection>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_multivariate_event_collections(page_params).await?;
                Ok((resp.multivariate_contracts, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for RFQs using cursor pagination.
    pub async fn get_rfqs_all(&self, params: GetRFQsParams) -> Result<Vec<RFQ>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_rfqs(page_params).await?;
                Ok((resp.rfqs, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for quotes using cursor pagination.
    pub async fn get_quotes_all(&self, params: GetQuotesParams) -> Result<Vec<Quote>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_quotes(page_params).await?;
                Ok((resp.quotes, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for structured targets using cursor pagination.
    pub async fn get_structured_targets_all(
        &self,
        params: GetStructuredTargetsParams,
    ) -> Result<Vec<StructuredTarget>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_structured_targets(page_params).await?;
                Ok((resp.structured_targets, resp.cursor))
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::TryStreamExt;
    use reqwest::Method;
    use reqwest::StatusCode;
    use serde_json::json;
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
        let body = br#"{"error":{"code":"bad_request","message":"invalid","service":"trade-api"}}"#;
        let err = build_http_error(StatusCode::BAD_REQUEST, body, None);
        match err {
            KalshiError::Http { api_error, .. } => {
                let api_error = api_error.expect("expected api error");
                assert_eq!(api_error.code.as_deref(), Some("bad_request"));
                assert_eq!(api_error.message.as_deref(), Some("invalid"));
                assert_eq!(api_error.service.as_deref(), Some("trade-api"));
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
