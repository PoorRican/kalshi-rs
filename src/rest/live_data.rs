//! Incentives, live data feeds, game stats, and milestones.
//!
//! `live_data` endpoints expose real-time feeds tied to sporting-event
//! milestones (scores, play-by-play). `milestones` endpoints enumerate the
//! milestones themselves. `incentive_programs` lists maker-rebate programs.

use crate::KalshiError;
use crate::rest::client::KalshiRestClient;
use crate::rest::events::Milestone;
use crate::rest::pagination::{CursorPager, stream_items};
use crate::types::{FixedPointCount, deserialize_null_as_empty_vec};
use futures::stream::Stream;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetIncentiveProgramsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub incentive_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetIncentiveProgramsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub incentive_programs: Vec<IncentiveProgram>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IncentiveProgram {
    pub id: String,
    pub market_id: String,
    pub market_ticker: String,
    pub incentive_type: String,
    pub start_date: String,
    pub end_date: String,
    pub period_reward: i64,
    pub paid_out: bool,
    #[serde(default)]
    pub discount_factor_bps: Option<i32>,
    #[serde(default)]
    pub target_size: Option<i32>,
    #[serde(default)]
    pub target_size_fp: Option<FixedPointCount>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetLiveDatasParams {
    pub milestone_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetLiveDatasResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub live_datas: Vec<LiveData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetLiveDataResponse {
    pub live_data: LiveData,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetLiveDataByMilestoneParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_player_stats: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiveData {
    #[serde(rename = "type")]
    pub live_data_type: String,
    #[serde(default)]
    pub details: Map<String, Value>,
    pub milestone_id: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

/// Live data for an event, keyed by event ticker. Added 2026-07-30.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventLiveData {
    #[serde(rename = "type")]
    pub live_data_type: String,
    #[serde(default)]
    pub details: Map<String, Value>,
    #[serde(default)]
    pub is_historical: Option<bool>,
    #[serde(default)]
    pub default_range: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub range_options: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetEventLiveDataResponse {
    pub live_data: EventLiveData,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEventLiveDataParams {
    /// Optional chart range hint (e.g. `15min`, `1h`, `1d`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
}

/// Per-station audit reading on a `WeatherIndexPoint`. Added 2026-08-20.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeatherIndexStationReading {
    pub station_id: String,
    pub code: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub temp_f: Option<f64>,
    #[serde(default)]
    pub obs_time_ms: Option<i64>,
    #[serde(default)]
    pub received_at_ms: Option<i64>,
    #[serde(default)]
    pub primary_code: Option<String>,
}

/// One minute of the Kalshi-computed city temperature index. Added 2026-08-20.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeatherIndexPoint {
    /// Event minute, unix milliseconds UTC.
    pub t: i64,
    /// Published index value, Fahrenheit rounded to 0.01. Absent on `incomplete` points.
    #[serde(default)]
    pub v: Option<f64>,
    /// `normal`, `degraded`, or (with `detailed=true`) `incomplete`.
    pub status: String,
    #[serde(default)]
    pub contributors: Option<i64>,
    /// Present only on points from the historical backfill seeding a newly listed city.
    #[serde(default)]
    pub receipt_basis: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub stations: Vec<WeatherIndexStationReading>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetWeatherIndexResponse {
    pub city: String,
    #[serde(default)]
    pub config_version: Option<String>,
    /// Always `fahrenheit`.
    pub units: String,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub timeseries: Vec<WeatherIndexPoint>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetWeatherIndexParams {
    /// Window start, unix milliseconds (inclusive). Must be paired with `to`
    /// unless `last_sec` is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    /// Window end, unix milliseconds (inclusive). Defaults to now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
    /// Trailing window in seconds; mutually exclusive with `from`/`to`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sec: Option<i64>,
    /// Include per-station audit readings on every point.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed: Option<bool>,
}

/// One configured member station in a weather-index calibration record. Added 2026-08-31.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeatherIndexCalibrationStation {
    pub station_id: String,
    pub weight: f64,
    pub offset_c: f64,
    #[serde(default)]
    pub update_note: Option<String>,
}

/// One published weather-index configuration record. Added 2026-08-31.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeatherIndexCalibration {
    pub config_version: String,
    #[serde(default)]
    pub published_at_ms: Option<i64>,
    pub effective_at_ms: i64,
    #[serde(default)]
    pub change_reason: Option<String>,
    #[serde(default)]
    pub calibration_window_start_ms: Option<i64>,
    #[serde(default)]
    pub calibration_window_end_ms: Option<i64>,
    pub city_reference_c: f64,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub stations: Vec<WeatherIndexCalibrationStation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetWeatherIndexCalibrationsResponse {
    pub city: String,
    /// Always `celsius`.
    pub units: String,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub calibrations: Vec<WeatherIndexCalibration>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetGameStatsResponse {
    #[serde(default)]
    pub pbp: Option<Value>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMilestonesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub milestone_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetMilestonesResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub milestones: Vec<Milestone>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetMilestoneResponse {
    pub milestone: Milestone,
}

impl KalshiRestClient {
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

    pub async fn get_live_data_by_milestone(
        &self,
        milestone_id: &str,
        params: GetLiveDataByMilestoneParams,
    ) -> Result<GetLiveDataResponse, KalshiError> {
        let path = Self::full_path(&format!("/live_data/milestone/{milestone_id}"));
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get live data for an event by ticker (crypto price charts, commodity price
    /// timeseries, weather observations, etc.). Added 2026-07-30.
    pub async fn get_event_live_data(
        &self,
        event_ticker: &str,
        params: GetEventLiveDataParams,
    ) -> Result<GetEventLiveDataResponse, KalshiError> {
        let path = Self::full_path(&format!("/live_data/events/{event_ticker}"));
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get the Kalshi-computed city temperature index (canonical minute-resolution
    /// series behind hourly temperature markets). Added 2026-08-20.
    pub async fn get_weather_index(
        &self,
        city: &str,
        params: GetWeatherIndexParams,
    ) -> Result<GetWeatherIndexResponse, KalshiError> {
        let path = Self::full_path(&format!("/live_data/weather/{city}"));
        self.send(
            Method::GET,
            &path,
            Some(&params),
            Option::<&()>::None,
            false,
        )
        .await
    }

    /// Get a city's published weather-index configuration timeline. Added 2026-08-31.
    pub async fn get_weather_index_calibrations(
        &self,
        city: &str,
    ) -> Result<GetWeatherIndexCalibrationsResponse, KalshiError> {
        let path = Self::full_path(&format!("/live_data/weather/{city}/calibrations"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            false,
        )
        .await
    }

    pub async fn get_game_stats(
        &self,
        milestone_id: &str,
    ) -> Result<GetGameStatsResponse, KalshiError> {
        let path = Self::full_path(&format!("/live_data/milestone/{milestone_id}/game_stats"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
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

    /// Stream milestones one by one.
    pub fn stream_milestones(
        &self,
        params: GetMilestonesParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Milestone, KalshiError>> + Send {
        stream_items(self.milestones_pager(params), max_items)
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
}
