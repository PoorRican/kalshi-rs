//! Trades and historical trade/order/fill endpoints.
//!
//! Public trade feed plus the `/historical/*` family of endpoints that return
//! archived fills, orders, markets, and the data-cutoff timestamps that
//! separate live from historical datasets.

use crate::KalshiError;
use crate::rest::client::KalshiRestClient;
use crate::rest::orders::GetOrdersResponse;
use crate::rest::pagination::{CursorPager, stream_items};
use crate::rest::portfolio::{GetFillsResponse, GetPositionsResponse};
use crate::types::{
    BookSide, FixedPointCount, FixedPointDollars, MveFilter, TradeTakerSide,
    deserialize_null_as_empty_vec,
};
use futures::stream::Stream;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Trade {
    pub trade_id: String,
    pub ticker: String,
    pub count_fp: FixedPointCount,
    pub yes_price_dollars: FixedPointDollars,
    pub no_price_dollars: FixedPointDollars,
    /// Deprecated 2026-05-07. Use `taker_outcome_side` / `taker_book_side`.
    /// Optional to tolerate eventual removal by the exchange.
    #[serde(default)]
    pub taker_side: Option<TradeTakerSide>,
    /// Normalized taker outcome side (yes | no). Added 2026-05-07.
    #[serde(default)]
    pub taker_outcome_side: Option<TradeTakerSide>,
    /// Normalized taker book side (bid | ask). Added 2026-05-07.
    #[serde(default)]
    pub taker_book_side: Option<BookSide>,
    pub created_time: String,
    /// True for block trades matched off-book (e.g. via RFQ). Added 2026-05-29.
    /// Defaults to `false` for payloads predating this field.
    #[serde(default)]
    pub is_block_trade: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetTradesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by block trade status. Omit to return all trades.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_block_trade: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetTradesResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub trades: Vec<Trade>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetHistoricalMarketsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tickers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_filter: Option<MveFilter>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetHistoricalFillsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetHistoricalOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// GET /historical/positions query params. Added 2026-07-23; `subaccount`
/// filter added 2026-09-03.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetHistoricalPositionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetHistoricalCutoffResponse {
    pub market_settled_ts: String,
    pub trades_created_ts: String,
    pub orders_updated_ts: String,
    /// Cutoff for settled positions archived from the live data set. Use
    /// `get_historical_positions` for positions older than this. Added 2026-07-23.
    #[serde(default)]
    pub market_positions_last_updated_ts: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl KalshiRestClient {
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

    /// List historical fills. Requires auth.
    pub async fn get_fills_historical(
        &self,
        params: GetHistoricalFillsParams,
    ) -> Result<GetFillsResponse, KalshiError> {
        self.get_historical_fills(params).await
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

    /// List settled positions archived to the historical database. Positions
    /// are archived per whole event; unsettled positions remain available via
    /// `get_positions`. Requires auth.
    pub async fn get_historical_positions(
        &self,
        params: GetHistoricalPositionsParams,
    ) -> Result<GetPositionsResponse, KalshiError> {
        let path = Self::full_path("/historical/positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
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

    /// List historical trades.
    pub async fn get_trades_historical(
        &self,
        params: GetTradesParams,
    ) -> Result<GetTradesResponse, KalshiError> {
        let path = Self::full_path("/historical/trades");
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
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

    /// Stream trades one by one.
    pub fn stream_trades(
        &self,
        params: GetTradesParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Trade, KalshiError>> + Send {
        stream_items(self.trades_pager(params), max_items)
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
}
