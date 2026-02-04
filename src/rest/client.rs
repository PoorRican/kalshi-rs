use crate::{KalshiAuth, KalshiEnvironment, KalshiError, REST_PREFIX};
use crate::rest::types::*;
use crate::types::ErrorResponse;

use futures::future::BoxFuture;
use futures::stream::{self, Stream};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use url::Url;

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub read_rps: u32,
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

#[derive(Debug, Clone, Copy)]
pub enum RateLimitTier {
    Basic,
}

impl RateLimitTier {
    fn config(self) -> RateLimitConfig {
        match self {
            RateLimitTier::Basic => RateLimitConfig::default(),
        }
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
    let raw_body = String::from_utf8_lossy(bytes).to_string();
    let api_error = serde_json::from_slice::<ErrorResponse>(bytes).ok();
    KalshiError::Http {
        status,
        api_error,
        raw_body,
        request_id,
    }
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
        let scheduled = if *last + interval > now { *last + interval } else { now };
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
/// # use kalshi::{KalshiEnvironment, KalshiRestClient, GetMarketsParams};
/// # async fn example() -> Result<(), kalshi::KalshiError> {
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
        dyn FnMut(Option<String>) -> BoxFuture<'static, Result<(Vec<T>, Option<String>), KalshiError>>
            + Send,
    >,
}

impl<T> CursorPager<T> {
    pub fn new<F>(cursor: Option<String>, fetch: F) -> Self
    where
        F: FnMut(Option<String>) -> BoxFuture<'static, Result<(Vec<T>, Option<String>), KalshiError>>
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
        if let Some(remaining) = state.remaining {
            if remaining == 0 {
                return None;
            }
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

#[derive(Debug, Clone)]
pub struct KalshiRestClient {
    http: Client,
    rest_origin: Url,
    auth: Option<KalshiAuth>,
    rate_limiter: Arc<RateLimiter>,
}

impl KalshiRestClient {
    pub fn new(env: KalshiEnvironment) -> Self {
        Self {
            http: Client::new(),
            rest_origin: env.rest_origin,
            auth: None,
            rate_limiter: Arc::new(RateLimiter::new(RateLimitConfig::default())),
        }
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
            HeaderValue::from_str(&h.timestamp_ms).map_err(|e| KalshiError::Header(e.to_string()))?,
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
        let mut headers = HeaderMap::new();

        if require_auth {
            let auth = self.auth.as_ref().ok_or(KalshiError::AuthRequired("REST endpoint"))?;
            // IMPORTANT: sign the path without query parameters
            Self::insert_auth_headers(&mut headers, auth, &method, full_path)?;
        }

        self.rate_limiter.wait(rate_limit_kind(&method)).await;

        let mut req = self.http.request(method, url).headers(headers);

        if let Some(q) = query {
            req = req.query(q);
        }
        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let request_id = resp
            .headers()
            .get("x-request-id")
            .or_else(|| resp.headers().get("request-id"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(build_http_error(status, &bytes, request_id));
        }

        let body_bytes = if bytes.is_empty() { b"{}" } else { bytes.as_ref() };
        Ok(serde_json::from_slice::<T>(body_bytes)?)
    }

    // ----------------------------
    // Public "market data" endpoints
    // ----------------------------

    /// GET /series
    pub async fn get_series_list(&self, params: GetSeriesListParams) -> Result<GetSeriesListResponse, KalshiError> {
        let path = Self::full_path("/series");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /series/{series_ticker}
    pub async fn get_series(&self, series_ticker: &str) -> Result<GetSeriesResponse, KalshiError> {
        let path = Self::full_path(&format!("/series/{series_ticker}"));
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, false).await
    }

    /// GET /events  (excludes multivariate events)
    pub async fn get_events(&self, params: GetEventsParams) -> Result<GetEventsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/events");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /events/{event_ticker}
    pub async fn get_event(&self, event_ticker: &str, with_nested_markets: Option<bool>) -> Result<GetEventResponse, KalshiError> {
        let path = Self::full_path(&format!("/events/{event_ticker}"));
        let params = GetEventParams { with_nested_markets };
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /markets
    pub async fn get_markets(&self, params: GetMarketsParams) -> Result<GetMarketsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/markets");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /markets/{ticker}
    pub async fn get_market(&self, market_ticker: &str) -> Result<GetMarketResponse, KalshiError> {
        let path = Self::full_path(&format!("/markets/{market_ticker}"));
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, false).await
    }

    /// GET /markets/{ticker}/orderbook
    pub async fn get_market_orderbook(
        &self,
        market_ticker: &str,
        depth: Option<u32>,
    ) -> Result<GetMarketOrderbookResponse, KalshiError> {
        let path = Self::full_path(&format!("/markets/{market_ticker}/orderbook"));
        let params = GetMarketOrderbookParams { depth };
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /markets/trades
    pub async fn get_trades(&self, params: GetTradesParams) -> Result<GetTradesResponse, KalshiError> {
        let path = Self::full_path("/markets/trades");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /exchange/status
    pub async fn get_exchange_status(&self) -> Result<GetExchangeStatusResponse, KalshiError> {
        let path = Self::full_path("/exchange/status");
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, false).await
    }

    /// GET /exchange/announcements
    pub async fn get_exchange_announcements(&self) -> Result<GetExchangeAnnouncementsResponse, KalshiError> {
        let path = Self::full_path("/exchange/announcements");
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, false).await
    }

    /// GET /exchange/schedule
    pub async fn get_exchange_schedule(&self) -> Result<GetExchangeScheduleResponse, KalshiError> {
        let path = Self::full_path("/exchange/schedule");
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, false).await
    }

    /// GET /exchange/user_data_timestamp
    pub async fn get_user_data_timestamp(&self) -> Result<GetUserDataTimestampResponse, KalshiError> {
        let path = Self::full_path("/exchange/user_data_timestamp");
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, false).await
    }

    /// GET /series/fee_changes
    pub async fn get_series_fee_changes(
        &self,
        params: GetSeriesFeeChangesParams,
    ) -> Result<GetSeriesFeeChangesResponse, KalshiError> {
        let path = Self::full_path("/series/fee_changes");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    // ----------------------------
    // Authenticated endpoints (portfolio / orders)
    // ----------------------------

    /// GET /portfolio/balance
    pub async fn get_balance(&self) -> Result<GetBalanceResponse, KalshiError> {
        let path = Self::full_path("/portfolio/balance");
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, true).await
    }

    /// GET /portfolio/positions
    pub async fn get_positions(&self, params: GetPositionsParams) -> Result<GetPositionsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/portfolio/positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true).await
    }

    /// GET /portfolio/orders
    pub async fn get_orders(&self, params: GetOrdersParams) -> Result<GetOrdersResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/portfolio/orders");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true).await
    }

    /// POST /portfolio/orders
    pub async fn create_order(&self, body: CreateOrderRequest) -> Result<CreateOrderResponse, KalshiError> {
        let path = Self::full_path("/portfolio/orders");
        body.validate()?;
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true).await
    }

    /// DELETE /portfolio/orders/{order_id}  (optional `subaccount` query param)
    pub async fn cancel_order(&self, order_id: &str, params: CancelOrderParams) -> Result<CancelOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}"));
        self.send(Method::DELETE, &path, Some(&params), Option::<&()>::None, true).await
    }

    /// GET /portfolio/fills
    pub async fn get_fills(&self, params: GetFillsParams) -> Result<GetFillsResponse, KalshiError> {
        let path = Self::full_path("/portfolio/fills");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true).await
    }

    /// GET /portfolio/settlements
    pub async fn get_settlements(&self, params: GetSettlementsParams) -> Result<GetSettlementsResponse, KalshiError> {
        let path = Self::full_path("/portfolio/settlements");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true).await
    }

    /// GET /account/limits
    pub async fn get_account_api_limits(&self) -> Result<GetAccountApiLimitsResponse, KalshiError> {
        let path = Self::full_path("/account/limits");
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, true).await
    }

    /// POST /portfolio/subaccounts
    pub async fn create_subaccount(&self) -> Result<CreateSubaccountResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts");
        self.send(Method::POST, &path, Option::<&()>::None, Option::<&()>::None, true).await
    }

    /// GET /portfolio/subaccounts/balances
    pub async fn get_subaccount_balances(&self) -> Result<GetSubaccountBalancesResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts/balances");
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, true).await
    }

    /// POST /portfolio/subaccounts/transfer
    pub async fn transfer_subaccount(
        &self,
        body: ApplySubaccountTransferRequest,
    ) -> Result<ApplySubaccountTransferResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts/transfer");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true).await
    }

    /// GET /portfolio/subaccounts/transfers
    pub async fn get_subaccount_transfers(
        &self,
        params: GetSubaccountTransfersParams,
    ) -> Result<GetSubaccountTransfersResponse, KalshiError> {
        let path = Self::full_path("/portfolio/subaccounts/transfers");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true).await
    }

    /// Generic cursor pagination helper.
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

    /// Create a pager for iterating over events page by page.
    ///
    /// # Example
    /// ```no_run
    /// # use kalshi::{KalshiEnvironment, KalshiRestClient, GetEventsParams};
    /// # async fn example() -> Result<(), kalshi::KalshiError> {
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

    /// Create a pager for iterating over markets page by page.
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

    /// Create a pager for iterating over trades page by page.
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

    /// Create a pager for iterating over positions page by page (authenticated).
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

    /// Create a pager for iterating over orders page by page (authenticated).
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

    /// Create a pager for iterating over fills page by page (authenticated).
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

    /// Create a pager for iterating over settlements page by page (authenticated).
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

    /// Create a pager for iterating over subaccount transfers page by page (authenticated).
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

    /// Stream events one by one.
    ///
    /// # Example
    /// ```no_run
    /// # use kalshi::{KalshiEnvironment, KalshiRestClient, GetEventsParams};
    /// # use futures::stream::TryStreamExt;
    /// # async fn example() -> Result<(), kalshi::KalshiError> {
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

    /// Stream positions one by one (authenticated).
    pub fn stream_positions(
        &self,
        params: GetPositionsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<PositionsPage, KalshiError>> + Send {
        stream_items(self.positions_pager(params), max_items)
    }

    /// Stream orders one by one (authenticated).
    pub fn stream_orders(
        &self,
        params: GetOrdersParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Order, KalshiError>> + Send {
        stream_items(self.orders_pager(params), max_items)
    }

    /// Stream fills one by one (authenticated).
    pub fn stream_fills(
        &self,
        params: GetFillsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Fill, KalshiError>> + Send {
        stream_items(self.fills_pager(params), max_items)
    }

    /// Stream settlements one by one (authenticated).
    pub fn stream_settlements(
        &self,
        params: GetSettlementsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Settlement, KalshiError>> + Send {
        stream_items(self.settlements_pager(params), max_items)
    }

    /// Stream subaccount transfers one by one (authenticated).
    pub fn stream_subaccount_transfers(
        &self,
        params: GetSubaccountTransfersParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<SubaccountTransfer, KalshiError>> + Send {
        stream_items(self.subaccount_transfers_pager(params), max_items)
    }

    /// Fetch all pages for markets using cursor pagination.
    pub async fn get_markets_all(&self, params: GetMarketsParams) -> Result<Vec<Market>, KalshiError> {
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
    pub async fn get_events_all(&self, params: GetEventsParams) -> Result<Vec<EventData>, KalshiError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::TryStreamExt;
    use reqwest::StatusCode;
    use std::sync::Arc;
    use tokio::time::{timeout, Duration};

    #[test]
    fn http_error_parses_json_body() {
        let body = br#"{"code":"rate_limit","message":"too fast"}"#;
        let err = build_http_error(StatusCode::TOO_MANY_REQUESTS, body, Some("req-1".to_string()));
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

    #[tokio::test]
    async fn rate_limiter_zero_rps_returns_quickly() {
        let limiter = RateLimiter::new(RateLimitConfig {
            read_rps: 0,
            write_rps: 0,
        });

        timeout(Duration::from_millis(10), limiter.wait(RateLimitKind::Read))
            .await
            .expect("read wait timed out");
        timeout(Duration::from_millis(10), limiter.wait(RateLimitKind::Write))
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
        let mut pages = VecDeque::from(vec![
            (vec![1, 2], Some("c1".to_string())),
            (vec![3], None),
        ]);
        let mut pager = CursorPager::new(None, move |_cursor| {
            let page = pages
                .pop_front()
                .unwrap_or((Vec::<i32>::new(), None));
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
            let page = pages
                .pop_front()
                .unwrap_or((Vec::<i32>::new(), None));
            Box::pin(async move { Ok(page) })
        });

        let items: Vec<i32> = stream_items(pager, Some(3))
            .try_collect()
            .await
            .unwrap();

        assert_eq!(items, vec![1, 2, 3]);
        // Should only fetch as many pages as needed to reach 3 items.
        assert_eq!(call_count.load(std::sync::atomic::Ordering::Relaxed), 2);
    }
}
