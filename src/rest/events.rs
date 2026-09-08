//! Event endpoints: regular events, multivariate events, forecast percentile
//! history, and event-level candlesticks.

use crate::KalshiError;
use crate::rest::client::KalshiRestClient;
use crate::rest::markets::{Market, MarketCandlestick};
use crate::rest::pagination::{CursorPager, stream_items};
use crate::rest::series::{EventMetadata, SettlementSource};
use crate::types::{EventStatus, deserialize_null_as_empty_vec, serialize_csv_opt};
use futures::stream::Stream;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// GET /events query params
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEventsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 200, max 200
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>, // default false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_milestones: Option<bool>, // default false

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EventStatus>, // open|closed|settled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_close_ts: Option<i64>, // seconds since epoch

    /// Event tickers comma-separated. Added 2026-06-18.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_csv_opt"
    )]
    pub tickers: Option<Vec<String>>,
}

impl GetEventsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit
            && (limit == 0 || limit > 200)
        {
            return Err(KalshiError::InvalidParams(
                "GET /events: limit must be 1..=200".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Milestone {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default, rename = "type")]
    pub milestone_type: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub related_event_tickers: Vec<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub notification_message: Option<String>,
    #[serde(default)]
    pub details: Option<Map<String, Value>>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub primary_event_tickers: Vec<String>,
    #[serde(default)]
    pub last_updated_ts: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub source_ids: Option<Map<String, Value>>,
    // Legacy/alternative shape fields still observed in some payloads.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ts: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventData {
    pub event_ticker: String,
    #[serde(default)]
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub collateral_return_type: Option<String>,
    #[serde(default)]
    pub mutually_exclusive: Option<bool>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub sub_title: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub strike_date: Option<String>,
    #[serde(default)]
    pub strike_period: Option<String>,
    #[serde(default)]
    pub last_updated_ts: Option<String>,
    #[serde(default)]
    pub occurrence_datetime: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub can_trade: Option<bool>,
    #[serde(default)]
    pub can_settle: Option<bool>,
    #[serde(default)]
    pub start_ts: Option<i64>,
    #[serde(default)]
    pub close_ts: Option<i64>,
    #[serde(default)]
    pub settled_ts: Option<i64>,
    #[serde(default)]
    pub series_id: Option<i64>,
    #[serde(default)]
    pub mutual_exclusive_group_id: Option<String>,
    #[serde(default)]
    pub mutual_exclusive_group_ids: Option<Vec<String>>,
    #[serde(default)]
    pub event_delta: Option<i64>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub volume_fp: Option<String>,
    #[serde(default)]
    pub markets: Option<Vec<Market>>,
    #[serde(default)]
    pub milestones: Option<Vec<Milestone>>,
    #[serde(default)]
    pub custom_strike: Option<Map<String, Value>>,
    #[serde(default)]
    pub product_metadata: Option<EventMetadata>,
    /// Official sources used to determine the event's markets. Mirrors the field
    /// already available on `Series`. Added 2026-06-18.
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub settlement_sources: Vec<SettlementSource>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetEventsResponse {
    pub events: Vec<EventData>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub milestones: Vec<Milestone>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEventParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>, // default false
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetEventResponse {
    pub event: EventData,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub markets: Vec<Market>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMultivariateEventsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetMultivariateEventsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub events: Vec<EventData>,
    #[serde(default)]
    pub cursor: Option<String>,
}

pub type GetEventMetadataResponse = EventMetadata;

#[derive(Debug, Clone, Serialize)]
pub struct GetEventForecastPercentileHistoryParams {
    pub percentiles: Vec<u32>,
    pub start_ts: i64,
    pub end_ts: i64,
    pub period_interval: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetEventForecastPercentilesHistoryResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub forecast_history: Vec<ForecastPercentilesPoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForecastPercentilesPoint {
    pub event_ticker: String,
    pub end_period_ts: i64,
    pub period_interval: i32,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub percentile_points: Vec<PercentilePoint>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PercentilePoint {
    pub percentile: i32,
    pub raw_numerical_forecast: f64,
    pub numerical_forecast: f64,
    pub formatted_forecast: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetEventCandlesticksParams {
    pub start_ts: i64,
    pub end_ts: i64,
    pub period_interval: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetEventCandlesticksResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub market_tickers: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub market_candlesticks: Vec<Vec<MarketCandlestick>>,
    pub adjusted_end_ts: i64,
}

impl KalshiRestClient {
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

    pub async fn get_event_forecast_percentile_history(
        &self,
        series_ticker: &str,
        ticker: &str,
        params: GetEventForecastPercentileHistoryParams,
    ) -> Result<GetEventForecastPercentilesHistoryResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/series/{series_ticker}/events/{ticker}/forecast_percentile_history"
        ));
        let query = Self::event_forecast_percentile_history_query(&params);
        self.send(Method::GET, &path, Some(&query), Option::<&()>::None, true)
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

    /// Create a pager for iterating over events page by page.
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

    /// Stream events one by one.
    pub fn stream_events(
        &self,
        params: GetEventsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<EventData, KalshiError>> + Send {
        stream_items(self.events_pager(params), max_items)
    }

    /// Stream multivariate events one by one.
    pub fn stream_multivariate_events(
        &self,
        params: GetMultivariateEventsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<EventData, KalshiError>> + Send {
        stream_items(self.multivariate_events_pager(params), max_items)
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
}
