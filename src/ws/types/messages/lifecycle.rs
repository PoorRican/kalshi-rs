use crate::rest::PriceRange;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// Market lifecycle message (type: "market_lifecycle_v2")
#[derive(Debug, Clone, Deserialize)]
pub struct WsMarketLifecycleV2 {
    pub market_ticker: String,
    #[serde(default)]
    pub event_type: Option<WsMarketLifecycleEventType>,
    /// Exchange shard the market lives on. Present only on `created` events. Added 2026-07-30.
    #[serde(default)]
    pub exchange_index: Option<i64>,
    #[serde(default)]
    pub open_ts: Option<i64>,
    #[serde(default)]
    pub close_ts: Option<i64>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub determination_ts: Option<i64>,
    #[serde(default)]
    pub settlement_value: Option<String>,
    #[serde(default)]
    pub settled_ts: Option<i64>,
    #[serde(default)]
    pub is_deactivated: Option<bool>,
    #[serde(default)]
    pub price_level_structure: Option<String>,
    /// Valid price bands, emitted alongside `price_level_structure` on `created`
    /// and `price_level_structure_updated` events. Added 2026-07-02.
    #[serde(default)]
    pub price_ranges: Option<Vec<PriceRange>>,
    /// Top-level updated floor strike. Per the AsyncAPI this key exists **only**
    /// on `metadata_updated` events and is distinct from
    /// `additional_metadata.floor_strike` (which is emitted on market creation).
    #[serde(default)]
    pub floor_strike: Option<f64>,
    /// Top-level updated strike type. Present only on `metadata_updated` events. Added 2026-06-18.
    #[serde(default)]
    pub strike_type: Option<String>,
    /// Top-level updated cap strike. Present only on `metadata_updated` events. Added 2026-06-18.
    #[serde(default)]
    pub cap_strike: Option<f64>,
    /// Top-level updated custom strike. Present only on `metadata_updated` events
    /// for custom/structured markets. Added 2026-06-18.
    #[serde(default)]
    pub custom_strike: Option<Map<String, Value>>,
    /// Top-level updated yes subtitle. Per the AsyncAPI this key exists **only**
    /// on `metadata_updated` events.
    #[serde(default)]
    pub yes_sub_title: Option<String>,
    #[serde(default)]
    pub additional_metadata: Option<WsMarketLifecycleAdditionalMetadata>,
    /// Catches any other top-level keys the exchange attaches to a lifecycle
    /// event (e.g. future `metadata_updated` fields beyond floor_strike /
    /// yes_sub_title).
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WsMarketLifecycleEventType {
    Created,
    Activated,
    Deactivated,
    CloseDateUpdated,
    Determined,
    Settled,
    PriceLevelStructureUpdated,
    /// Fires when market metadata (name, title, subtitles, etc.) changes. Added 2026-05-11.
    MetadataUpdated,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsMarketLifecycleAdditionalMetadata {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub yes_sub_title: Option<String>,
    #[serde(default)]
    pub no_sub_title: Option<String>,
    #[serde(default)]
    pub rules_primary: Option<String>,
    #[serde(default)]
    pub rules_secondary: Option<String>,
    #[serde(default)]
    pub can_close_early: Option<bool>,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub expected_expiration_ts: Option<i64>,
    #[serde(default)]
    pub strike_type: Option<String>,
    #[serde(default)]
    pub floor_strike: Option<f64>,
    #[serde(default)]
    pub cap_strike: Option<f64>,
    #[serde(default)]
    pub custom_strike: Option<BTreeMap<String, String>>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

/// Event lifecycle message (type: "event_lifecycle")
#[derive(Debug, Clone, Deserialize)]
pub struct WsEventLifecycle {
    pub event_ticker: String,
    /// Exchange shard the event's markets live on. Added 2026-07-30.
    #[serde(default)]
    pub exchange_index: Option<i64>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub collateral_return_type: Option<String>,
    #[serde(default)]
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub strike_date: Option<i64>,
    #[serde(default)]
    pub strike_period: Option<String>,
    #[serde(default)]
    pub additional_metadata: Option<WsEventLifecycleAdditionalMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsEventLifecycleAdditionalMetadata {
    #[serde(default)]
    pub custom_strike: Option<BTreeMap<String, String>>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

/// Market lifecycle message (type: "market_lifecycle_v2")
#[derive(Debug, Clone, Deserialize)]
pub struct WsMarketLifecycleV2Ref<'a> {
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(default)]
    pub event_type: Option<WsMarketLifecycleEventType>,
    #[serde(default)]
    pub exchange_index: Option<i64>,
    #[serde(default)]
    pub open_ts: Option<i64>,
    #[serde(default)]
    pub close_ts: Option<i64>,
    #[serde(default, borrow)]
    pub result: Option<Cow<'a, str>>,
    #[serde(default)]
    pub determination_ts: Option<i64>,
    #[serde(default, borrow)]
    pub settlement_value: Option<Cow<'a, str>>,
    #[serde(default)]
    pub settled_ts: Option<i64>,
    #[serde(default)]
    pub is_deactivated: Option<bool>,
    #[serde(default, borrow)]
    pub price_level_structure: Option<Cow<'a, str>>,
    #[serde(default)]
    pub price_ranges: Option<Vec<PriceRange>>,
    /// Top-level updated floor strike; present only on `metadata_updated` events.
    #[serde(default)]
    pub floor_strike: Option<f64>,
    #[serde(default, borrow)]
    pub strike_type: Option<Cow<'a, str>>,
    #[serde(default)]
    pub cap_strike: Option<f64>,
    #[serde(default)]
    pub custom_strike: Option<Map<String, Value>>,
    /// Top-level updated yes subtitle; present only on `metadata_updated` events.
    #[serde(default, borrow)]
    pub yes_sub_title: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub additional_metadata: Option<WsMarketLifecycleAdditionalMetadataRef<'a>>,
    /// Catches any other top-level lifecycle keys not modeled above.
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl<'a> WsMarketLifecycleV2Ref<'a> {
    pub fn into_owned(self) -> WsMarketLifecycleV2 {
        WsMarketLifecycleV2 {
            market_ticker: self.market_ticker.into_owned(),
            event_type: self.event_type,
            exchange_index: self.exchange_index,
            open_ts: self.open_ts,
            close_ts: self.close_ts,
            result: self.result.map(Cow::into_owned),
            determination_ts: self.determination_ts,
            settlement_value: self.settlement_value.map(Cow::into_owned),
            settled_ts: self.settled_ts,
            is_deactivated: self.is_deactivated,
            price_level_structure: self.price_level_structure.map(Cow::into_owned),
            price_ranges: self.price_ranges,
            floor_strike: self.floor_strike,
            strike_type: self.strike_type.map(Cow::into_owned),
            cap_strike: self.cap_strike,
            custom_strike: self.custom_strike,
            yes_sub_title: self.yes_sub_title.map(Cow::into_owned),
            additional_metadata: self
                .additional_metadata
                .map(WsMarketLifecycleAdditionalMetadataRef::into_owned),
            extra: self.extra,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsMarketLifecycleAdditionalMetadataRef<'a> {
    #[serde(default, borrow)]
    pub name: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub title: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub yes_sub_title: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub no_sub_title: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub rules_primary: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub rules_secondary: Option<Cow<'a, str>>,
    #[serde(default)]
    pub can_close_early: Option<bool>,
    #[serde(default, borrow)]
    pub event_ticker: Option<Cow<'a, str>>,
    #[serde(default)]
    pub expected_expiration_ts: Option<i64>,
    #[serde(default, borrow)]
    pub strike_type: Option<Cow<'a, str>>,
    #[serde(default)]
    pub floor_strike: Option<f64>,
    #[serde(default)]
    pub cap_strike: Option<f64>,
    #[serde(default)]
    pub custom_strike: Option<BTreeMap<String, String>>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl<'a> WsMarketLifecycleAdditionalMetadataRef<'a> {
    pub fn into_owned(self) -> WsMarketLifecycleAdditionalMetadata {
        WsMarketLifecycleAdditionalMetadata {
            name: self.name.map(Cow::into_owned),
            title: self.title.map(Cow::into_owned),
            yes_sub_title: self.yes_sub_title.map(Cow::into_owned),
            no_sub_title: self.no_sub_title.map(Cow::into_owned),
            rules_primary: self.rules_primary.map(Cow::into_owned),
            rules_secondary: self.rules_secondary.map(Cow::into_owned),
            can_close_early: self.can_close_early,
            event_ticker: self.event_ticker.map(Cow::into_owned),
            expected_expiration_ts: self.expected_expiration_ts,
            strike_type: self.strike_type.map(Cow::into_owned),
            floor_strike: self.floor_strike,
            cap_strike: self.cap_strike,
            custom_strike: self.custom_strike,
            extra: self.extra,
        }
    }
}

/// Event lifecycle message (type: "event_lifecycle")
#[derive(Debug, Clone, Deserialize)]
pub struct WsEventLifecycleRef<'a> {
    #[serde(borrow)]
    pub event_ticker: Cow<'a, str>,
    #[serde(default)]
    pub exchange_index: Option<i64>,
    #[serde(default, borrow)]
    pub title: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub subtitle: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub collateral_return_type: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub series_ticker: Option<Cow<'a, str>>,
    #[serde(default)]
    pub strike_date: Option<i64>,
    #[serde(default, borrow)]
    pub strike_period: Option<Cow<'a, str>>,
    #[serde(default)]
    pub additional_metadata: Option<WsEventLifecycleAdditionalMetadataRef>,
}

impl<'a> WsEventLifecycleRef<'a> {
    pub fn into_owned(self) -> WsEventLifecycle {
        WsEventLifecycle {
            event_ticker: self.event_ticker.into_owned(),
            exchange_index: self.exchange_index,
            title: self.title.map(Cow::into_owned),
            subtitle: self.subtitle.map(Cow::into_owned),
            collateral_return_type: self.collateral_return_type.map(Cow::into_owned),
            series_ticker: self.series_ticker.map(Cow::into_owned),
            strike_date: self.strike_date,
            strike_period: self.strike_period.map(Cow::into_owned),
            additional_metadata: self
                .additional_metadata
                .map(WsEventLifecycleAdditionalMetadataRef::into_owned),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsEventLifecycleAdditionalMetadataRef {
    #[serde(default)]
    pub custom_strike: Option<BTreeMap<String, String>>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

impl WsEventLifecycleAdditionalMetadataRef {
    pub fn into_owned(self) -> WsEventLifecycleAdditionalMetadata {
        WsEventLifecycleAdditionalMetadata {
            custom_strike: self.custom_strike,
            extra: self.extra,
        }
    }
}

/// Event fee update message (type: "event_fee_update").
///
/// Delivered on the `market_lifecycle_v2` channel when an event's fee override
/// is set or cleared. Both override fields are `null` when the override is
/// cleared. `fee_type_override` is kept as a raw string (values include
/// `quadratic`, `quadratic_with_maker_fees`, `flat`) so unknown variants are
/// preserved losslessly for fee math.
#[derive(Debug, Clone, Deserialize)]
pub struct WsEventFeeUpdate {
    pub event_ticker: String,
    #[serde(default)]
    pub fee_type_override: Option<String>,
    #[serde(default)]
    pub fee_multiplier_override: Option<f64>,
}

/// Event fee update message (type: "event_fee_update").
#[derive(Debug, Clone, Deserialize)]
pub struct WsEventFeeUpdateRef<'a> {
    #[serde(borrow)]
    pub event_ticker: Cow<'a, str>,
    #[serde(default, borrow)]
    pub fee_type_override: Option<Cow<'a, str>>,
    #[serde(default)]
    pub fee_multiplier_override: Option<f64>,
}

impl<'a> WsEventFeeUpdateRef<'a> {
    pub fn into_owned(self) -> WsEventFeeUpdate {
        WsEventFeeUpdate {
            event_ticker: self.event_ticker.into_owned(),
            fee_type_override: self.fee_type_override.map(Cow::into_owned),
            fee_multiplier_override: self.fee_multiplier_override,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Per the AsyncAPI, `metadata_updated` carries the updated values
    /// (`floor_strike`, `yes_sub_title`) at the top level of the payload, not
    /// nested under `additional_metadata`. They must not be silently dropped.
    #[test]
    fn metadata_updated_surfaces_top_level_fields() {
        let json = r#"{
            "event_type": "metadata_updated",
            "market_ticker": "KXHIGHNY-24JAN01-T60",
            "floor_strike": 60.5,
            "yes_sub_title": "Above 60°F",
            "some_future_key": "kept"
        }"#;

        let owned: WsMarketLifecycleV2 = serde_json::from_str(json).unwrap();
        assert_eq!(
            owned.event_type,
            Some(WsMarketLifecycleEventType::MetadataUpdated)
        );
        assert_eq!(owned.floor_strike, Some(60.5));
        assert_eq!(owned.yes_sub_title.as_deref(), Some("Above 60°F"));
        assert_eq!(
            owned.extra.get("some_future_key").and_then(Value::as_str),
            Some("kept")
        );

        // Borrowed path must round-trip to the same surfaced values.
        let borrowed: WsMarketLifecycleV2Ref = serde_json::from_str(json).unwrap();
        let round_tripped = borrowed.into_owned();
        assert_eq!(round_tripped.floor_strike, Some(60.5));
        assert_eq!(round_tripped.yes_sub_title.as_deref(), Some("Above 60°F"));
        assert_eq!(
            round_tripped
                .extra
                .get("some_future_key")
                .and_then(Value::as_str),
            Some("kept")
        );
    }
}
