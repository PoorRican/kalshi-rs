use crate::{KalshiAuth, KalshiEnvironment, KalshiError, REST_PREFIX};
use crate::types::*;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

#[derive(Debug, Clone)]
pub struct KalshiRestClient {
    http: Client,
    rest_origin: Url,
    auth: Option<KalshiAuth>,
}

impl KalshiRestClient {
    pub fn new(env: KalshiEnvironment) -> Self {
        Self {
            http: Client::new(),
            rest_origin: env.rest_origin,
            auth: None,
        }
    }

    /// Attach auth so you can call authenticated endpoints.
    pub fn with_auth(mut self, auth: KalshiAuth) -> Self {
        self.auth = Some(auth);
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
            // IMPORTANT: sign the path without query parameters :contentReference[oaicite:23]{index=23}
            Self::insert_auth_headers(&mut headers, auth, &method, full_path)?;
        }

        let mut req = self.http.request(method, url).headers(headers);

        if let Some(q) = query {
            req = req.query(q);
        }
        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send().await?.error_for_status()?;
        Ok(resp.json::<T>().await?)
    }

    // ----------------------------
    // Public "market data" endpoints
    // ----------------------------

    /// GET /series  :contentReference[oaicite:24]{index=24}
    pub async fn get_series_list(&self, params: GetSeriesListParams) -> Result<GetSeriesListResponse, KalshiError> {
        let path = Self::full_path("/series");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /events  (excludes multivariate events) :contentReference[oaicite:25]{index=25}
    pub async fn get_events(&self, params: GetEventsParams) -> Result<GetEventsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/events");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /events/{event_ticker}  :contentReference[oaicite:26]{index=26}
    pub async fn get_event(&self, event_ticker: &str, with_nested_markets: Option<bool>) -> Result<GetEventResponse, KalshiError> {
        let path = Self::full_path(&format!("/events/{event_ticker}"));
        let params = GetEventParams { with_nested_markets };
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /markets  :contentReference[oaicite:27]{index=27}
    pub async fn get_markets(&self, params: GetMarketsParams) -> Result<GetMarketsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/markets");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, false).await
    }

    /// GET /markets/{ticker}  :contentReference[oaicite:28]{index=28}
    pub async fn get_market(&self, market_ticker: &str) -> Result<GetMarketResponse, KalshiError> {
        let path = Self::full_path(&format!("/markets/{market_ticker}"));
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, false).await
    }

    // ----------------------------
    // Authenticated endpoints (portfolio / orders)
    // ----------------------------

    /// GET /portfolio/balance  :contentReference[oaicite:29]{index=29}
    pub async fn get_balance(&self) -> Result<GetBalanceResponse, KalshiError> {
        let path = Self::full_path("/portfolio/balance");
        self.send(Method::GET, &path, Option::<&()>::None, Option::<&()>::None, true).await
    }

    /// GET /portfolio/positions  :contentReference[oaicite:30]{index=30}
    pub async fn get_positions(&self, params: GetPositionsParams) -> Result<GetPositionsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/portfolio/positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true).await
    }

    /// GET /portfolio/orders  :contentReference[oaicite:31]{index=31}
    pub async fn get_orders(&self, params: GetOrdersParams) -> Result<GetOrdersResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/portfolio/orders");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true).await
    }

    /// POST /portfolio/orders  :contentReference[oaicite:32]{index=32}
    pub async fn create_order(&self, body: CreateOrderRequest) -> Result<CreateOrderResponse, KalshiError> {
        let path = Self::full_path("/portfolio/orders");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true).await
    }

    /// DELETE /portfolio/orders/{order_id}  (optional `subaccount` query param) :contentReference[oaicite:33]{index=33}
    pub async fn cancel_order(&self, order_id: &str, params: CancelOrderParams) -> Result<CancelOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}"));
        self.send(Method::DELETE, &path, Some(&params), Option::<&()>::None, true).await
    }
}

