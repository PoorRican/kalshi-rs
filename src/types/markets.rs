use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

use crate::error::KalshiError;
use super::{AnyJson, FixedPointDollars, serialize_csv_opt};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketState {
    Initialized,
    Inactive,
    Active,
    Closed,
    Determined,
    Disputed,
    Amended,
    Finalized,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum MarketStatus {
    Unopened,
    Open,
    Paused,
    Closed,
    Settled,
}

impl MarketStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            MarketStatus::Unopened => "unopened",
            MarketStatus::Open => "open",
            MarketStatus::Paused => "paused",
            MarketStatus::Closed => "closed",
            MarketStatus::Settled => "settled",
        }
    }
}

impl fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for MarketStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MveFilter {
    Only,
    Exclude,
}

impl MveFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            MveFilter::Only => "only",
            MveFilter::Exclude => "exclude",
        }
    }
}

impl fmt::Display for MveFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for MveFilter {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MveSelectedLeg {
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub market_ticker: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub yes_settlement_value_dollars: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceRange {
    #[serde(default)]
    pub min_price: Option<String>,
    #[serde(default)]
    pub max_price: Option<String>,
    #[serde(default)]
    pub increment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Market {
    pub ticker: String,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub market_id: Option<String>,
    #[serde(default)]
    pub status: Option<MarketState>,
    #[serde(default)]
    pub market_type: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub event_title: Option<String>,
    #[serde(default)]
    pub yes_sub_title: Option<String>,
    #[serde(default)]
    pub no_sub_title: Option<String>,
    #[serde(default)]
    pub rules_primary: Option<String>,
    #[serde(default)]
    pub rules_secondary: Option<String>,
    #[serde(default)]
    pub resolution_source: Option<String>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub can_trade: Option<bool>,
    #[serde(default)]
    pub can_settle: Option<bool>,
    #[serde(default)]
    pub can_close_early: Option<bool>,
    #[serde(default)]
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub series_id: Option<i64>,
    #[serde(default)]
    pub event_id: Option<i64>,
    #[serde(default)]
    pub response_price_units: Option<String>,
    #[serde(default)]
    pub price_level_structure: Option<String>,
    #[serde(default)]
    pub open_ts: Option<i64>,
    #[serde(default)]
    pub close_ts: Option<i64>,
    #[serde(default)]
    pub settled_ts: Option<i64>,
    #[serde(default)]
    pub expiration_ts: Option<i64>,
    #[serde(default)]
    pub open_time: Option<String>,
    #[serde(default)]
    pub close_time: Option<String>,
    #[serde(default)]
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub latest_expiration_time: Option<String>,
    #[serde(default)]
    pub created_ts: Option<i64>,
    #[serde(default)]
    pub updated_ts: Option<i64>,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub updated_time: Option<String>,
    #[serde(default)]
    pub floor_price: Option<i64>,
    #[serde(default)]
    pub cap_price: Option<i64>,
    #[serde(default)]
    pub yes_bid: Option<i64>,
    #[serde(default)]
    pub yes_ask: Option<i64>,
    #[serde(default)]
    pub no_bid: Option<i64>,
    #[serde(default)]
    pub no_ask: Option<i64>,
    #[serde(default)]
    pub price: Option<i64>,
    #[serde(default)]
    pub last_price: Option<i64>,
    #[serde(default)]
    pub yes_bid_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub yes_ask_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub no_bid_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub no_ask_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub last_price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub volume_fp: Option<String>,
    #[serde(default)]
    pub volume_24h: Option<i64>,
    #[serde(default)]
    pub volume_24h_fp: Option<String>,
    #[serde(default)]
    pub open_interest: Option<i64>,
    #[serde(default)]
    pub open_interest_fp: Option<String>,
    #[serde(default)]
    pub liquidity: Option<i64>,
    #[serde(default)]
    pub liquidity_fp: Option<String>,
    #[serde(default)]
    pub custom_strike: Option<AnyJson>,
    #[serde(default)]
    pub mve_selected_legs: Option<Vec<MveSelectedLeg>>,
    #[serde(default)]
    pub price_ranges: Option<Vec<AnyJson>>,
}

/// GET /markets query params and constraints
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMarketsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 100, max 1000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    /// Event tickers comma-separated (max 10)
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
    pub event_ticker: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_created_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_created_ts: Option<i64>,

    /// min_updated_ts is incompatible with any other filters besides mve_filter=exclude.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_updated_ts: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_close_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_close_ts: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_settled_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_settled_ts: Option<i64>,

    /// Only one status filter may be supplied at a time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MarketStatus>,

    /// Market tickers comma-separated.
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
    pub tickers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_filter: Option<MveFilter>,
}

impl GetMarketsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit {
            if limit == 0 || limit > 1000 {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: limit must be 1..=1000".to_string(),
                ));
            }
        }
        if let Some(evts) = &self.event_ticker {
            if evts.len() > 10 {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: event_ticker supports up to 10 tickers".to_string(),
                ));
            }
        }

        // Timestamp filter compatibility rules
        let created = self.min_created_ts.is_some() || self.max_created_ts.is_some();
        let close = self.min_close_ts.is_some() || self.max_close_ts.is_some();
        let settled = self.min_settled_ts.is_some() || self.max_settled_ts.is_some();
        let updated = self.min_updated_ts.is_some();

        let groups = [created, close, settled, updated]
            .iter()
            .filter(|x| **x)
            .count();
        if groups > 1 {
            return Err(KalshiError::InvalidParams(
                "GET /markets: timestamp filters are mutually exclusive (created vs close vs settled vs updated)"
                    .to_string(),
            ));
        }

        if updated {
            if self.status.is_some()
                || self.series_ticker.is_some()
                || self.event_ticker.is_some()
                || self.tickers.is_some()
                || created
                || close
                || settled
            {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: min_updated_ts cannot be combined with other filters (except mve_filter=exclude)"
                        .to_string(),
                ));
            }
            if matches!(self.mve_filter, Some(MveFilter::Only)) {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: with min_updated_ts, only mve_filter=exclude is allowed".to_string(),
                ));
            }
        }

        if created {
            if matches!(
                self.status,
                Some(MarketStatus::Closed | MarketStatus::Settled | MarketStatus::Paused)
            ) {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: created_ts filters are only compatible with status unopened/open or no status"
                        .to_string(),
                ));
            }
        }
        if close {
            if matches!(
                self.status,
                Some(MarketStatus::Unopened | MarketStatus::Open | MarketStatus::Settled | MarketStatus::Paused)
            ) {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: close_ts filters are only compatible with status closed or no status"
                        .to_string(),
                ));
            }
        }
        if settled {
            if matches!(
                self.status,
                Some(MarketStatus::Unopened | MarketStatus::Open | MarketStatus::Closed | MarketStatus::Paused)
            ) {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: settled_ts filters are only compatible with status settled or no status"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketsResponse {
    pub markets: Vec<Market>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketResponse {
    pub market: Market,
}
