use serde::{Deserialize, Serialize};

use super::AnyJson;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeeType {
    Quadratic,
    Flat,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettlementSource {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Series {
    pub ticker: String,
    #[serde(default)]
    pub frequency: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub subcategory: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub settlement_sources: Vec<SettlementSource>,
    #[serde(default)]
    pub contract_url: Option<String>,
    #[serde(default)]
    pub contract_terms_url: Option<String>,
    #[serde(default)]
    pub fee_type: Option<FeeType>,
    #[serde(default)]
    pub fee_multiplier: Option<i64>,
    #[serde(default)]
    pub additional_prohibitions: Vec<String>,
    #[serde(default)]
    pub product_metadata: Option<AnyJson>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub volume_fp: Option<String>,
    #[serde(default)]
    pub latest_event_ticker: Option<String>,
    #[serde(default)]
    pub inactive: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetSeriesListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Docs: "tags" is a string (not explicitly CSV-typed), so keep as raw string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_product_metadata: Option<bool>,
    /// If true, includes total volume traded across all events in each series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_volume: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesListResponse {
    pub series: Vec<Series>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesResponse {
    pub series: Series,
}
