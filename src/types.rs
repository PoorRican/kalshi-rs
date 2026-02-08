use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;

/// Serialize `Option<Vec<T>>` as a single comma-separated query param.
pub fn serialize_csv_opt<T, S>(value: &Option<Vec<T>>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: fmt::Display,
    S: Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(items) => {
            let s = items
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            serializer.serialize_str(&s)
        }
    }
}

/// Deserialize string or number into a String (fixed-point values often arrive as strings).
pub fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNumber;

    impl<'de> serde::de::Visitor<'de> for StringOrNumber {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.to_string())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.to_string())
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.to_string())
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.to_string())
        }
    }

    deserializer.deserialize_any(StringOrNumber)
}

/// Deserialize a null or array into a `Vec<T>` (null maps to empty vec).
pub fn deserialize_null_as_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<Vec<T>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Fixed-point dollar string (e.g. "0.5600").
pub type FixedPointDollars = String;

/// Fixed-point contract count string (e.g. "10.00").
pub type FixedPointCount = String;

/// Typed wrapper for arbitrary JSON payloads.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnyJson(pub Value);

impl AnyJson {
    pub fn as_value(&self) -> &Value {
        &self.0
    }
}

impl From<Value> for AnyJson {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl Serialize for AnyJson {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnyJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(Value::deserialize(deserializer)?))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorResponse {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub service: Option<String>,
}

/// --- Fee Type ---

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeeType {
    Quadratic,
    Flat,
    #[serde(other)]
    Unknown,
}

/// --- Event Status ---

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

/// --- Market Status ---

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatus {
    Unopened,
    Open,
    Paused,
    Closed,
    Settled,
    #[serde(other)]
    Unknown,
}

impl MarketStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            MarketStatus::Unopened => "unopened",
            MarketStatus::Open => "open",
            MarketStatus::Paused => "paused",
            MarketStatus::Closed => "closed",
            MarketStatus::Settled => "settled",
            MarketStatus::Unknown => "unknown",
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

/// --- MVE Filter ---

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

/// --- Position Count Filter ---

#[derive(Debug, Clone, Copy)]
pub enum PositionCountFilter {
    Position,
    TotalTraded,
}

impl PositionCountFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            PositionCountFilter::Position => "position",
            PositionCountFilter::TotalTraded => "total_traded",
        }
    }
}

impl fmt::Display for PositionCountFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// --- Order Status ---

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Resting,
    Canceled,
    Executed,
    #[serde(other)]
    Unknown,
}

impl OrderStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            OrderStatus::Resting => "resting",
            OrderStatus::Canceled => "canceled",
            OrderStatus::Executed => "executed",
            OrderStatus::Unknown => "unknown",
        }
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for OrderStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// --- Yes/No (Side) ---

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YesNo {
    Yes,
    No,
    #[serde(other)]
    Unknown,
}

impl Default for YesNo {
    fn default() -> Self {
        YesNo::Yes
    }
}

impl YesNo {
    pub fn as_str(self) -> &'static str {
        match self {
            YesNo::Yes => "yes",
            YesNo::No => "no",
            YesNo::Unknown => "unknown",
        }
    }
}

impl fmt::Display for YesNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for YesNo {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// --- Buy/Sell (Action) ---

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuySell {
    Buy,
    Sell,
    #[serde(other)]
    Unknown,
}

impl Default for BuySell {
    fn default() -> Self {
        BuySell::Buy
    }
}

impl BuySell {
    pub fn as_str(self) -> &'static str {
        match self {
            BuySell::Buy => "buy",
            BuySell::Sell => "sell",
            BuySell::Unknown => "unknown",
        }
    }
}

impl fmt::Display for BuySell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for BuySell {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// --- Order Type ---

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Limit,
    Market,
    #[serde(other)]
    Unknown,
}

impl OrderType {
    pub fn as_str(self) -> &'static str {
        match self {
            OrderType::Limit => "limit",
            OrderType::Market => "market",
            OrderType::Unknown => "unknown",
        }
    }
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for OrderType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// --- Time In Force ---

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    FillOrKill,
    GoodTillCanceled,
    ImmediateOrCancel,
    #[serde(other)]
    Unknown,
}

impl TimeInForce {
    pub fn as_str(self) -> &'static str {
        match self {
            TimeInForce::FillOrKill => "fill_or_kill",
            TimeInForce::GoodTillCanceled => "good_till_canceled",
            TimeInForce::ImmediateOrCancel => "immediate_or_cancel",
            TimeInForce::Unknown => "unknown",
        }
    }
}

impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for TimeInForce {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// --- Self Trade Prevention Type ---

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelfTradePreventionType {
    TakerAtCross,
    Maker,
    #[serde(other)]
    Unknown,
}

impl SelfTradePreventionType {
    pub fn as_str(self) -> &'static str {
        match self {
            SelfTradePreventionType::TakerAtCross => "taker_at_cross",
            SelfTradePreventionType::Maker => "maker",
            SelfTradePreventionType::Unknown => "unknown",
        }
    }
}

impl fmt::Display for SelfTradePreventionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for SelfTradePreventionType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// --- Trade Taker Side ---

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeTakerSide {
    Yes,
    No,
    #[serde(other)]
    Unknown,
}

impl TradeTakerSide {
    pub fn as_str(self) -> &'static str {
        match self {
            TradeTakerSide::Yes => "yes",
            TradeTakerSide::No => "no",
            TradeTakerSide::Unknown => "unknown",
        }
    }
}

impl fmt::Display for TradeTakerSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for TradeTakerSide {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_status_deserialize_known() {
        let status: MarketStatus = serde_json::from_str("\"open\"").unwrap();
        assert!(matches!(status, MarketStatus::Open));
    }

    #[test]
    fn market_status_deserialize_unknown() {
        let status: MarketStatus = serde_json::from_str("\"mystery\"").unwrap();
        assert!(matches!(status, MarketStatus::Unknown));
    }
}
