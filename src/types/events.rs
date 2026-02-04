use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

use crate::error::KalshiError;
use super::{AnyJson, Market};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Open,
    Closed,
    Settled,
    #[serde(other)]
    Unknown,
}

impl EventStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            EventStatus::Open => "open",
            EventStatus::Closed => "closed",
            EventStatus::Settled => "settled",
            EventStatus::Unknown => "unknown",
        }
    }
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for EventStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

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
    pub with_milestones: Option<bool>,     // default false

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EventStatus>,       // open|closed|settled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_close_ts: Option<i64>,         // seconds since epoch
}

impl GetEventsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit {
            if limit == 0 || limit > 200 {
                return Err(KalshiError::InvalidParams(
                    "GET /events: limit must be 1..=200".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Milestone {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ts: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventData {
    pub event_ticker: String,
    #[serde(default)]
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub sub_title: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
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
    pub custom_strike: Option<AnyJson>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventsResponse {
    pub events: Vec<EventData>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEventParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>, // default false
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventResponse {
    pub event: EventData,
}
