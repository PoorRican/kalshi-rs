use crate::types::{BookSide, BuySell, YesNo};
use serde::Deserialize;
use std::borrow::Cow;

/// Fill channel message (type: "fill")
#[derive(Debug, Clone, Deserialize)]
pub struct WsFill {
    pub trade_id: String,
    pub order_id: String,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(alias = "ticker")]
    pub market_ticker: String,
    /// Identifier for the exchange shard where the fill occurred. Added 2026-08-20.
    #[serde(default)]
    pub exchange_index: Option<u32>,
    /// Deprecated 2026-05-07; removed ~2026-05-28. Use `outcome_side`.
    #[serde(default)]
    pub side: Option<YesNo>,
    /// Deprecated 2026-05-07; removed ~2026-05-28. Use `book_side`.
    #[serde(default)]
    pub action: Option<BuySell>,
    /// Normalized outcome side (yes | no). Added 2026-05-07.
    #[serde(default)]
    pub outcome_side: Option<YesNo>,
    /// Normalized book side (bid | ask). Added 2026-05-07.
    #[serde(default)]
    pub book_side: Option<BookSide>,
    pub count_fp: String,
    pub yes_price_dollars: String,
    pub is_taker: bool,
    pub fee_cost: String,
    pub ts: i64,
    pub ts_ms: i64,
    pub post_position_fp: String,
    pub purchased_side: YesNo,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    #[serde(alias = "subaccount_number")]
    pub subaccount: Option<i64>,
}

/// Fill channel message (type: "fill")
#[derive(Debug, Clone, Deserialize)]
pub struct WsFillRef<'a> {
    #[serde(borrow)]
    pub trade_id: Cow<'a, str>,
    #[serde(borrow)]
    pub order_id: Cow<'a, str>,
    #[serde(default, borrow)]
    pub client_order_id: Option<Cow<'a, str>>,
    #[serde(alias = "ticker", borrow)]
    pub market_ticker: Cow<'a, str>,
    /// Identifier for the exchange shard where the fill occurred. Added 2026-08-20.
    #[serde(default)]
    pub exchange_index: Option<u32>,
    /// Deprecated 2026-05-07; removed ~2026-05-28. Use `outcome_side`.
    #[serde(default)]
    pub side: Option<YesNo>,
    /// Deprecated 2026-05-07; removed ~2026-05-28. Use `book_side`.
    #[serde(default)]
    pub action: Option<BuySell>,
    /// Normalized outcome side (yes | no). Added 2026-05-07.
    #[serde(default)]
    pub outcome_side: Option<YesNo>,
    /// Normalized book side (bid | ask). Added 2026-05-07.
    #[serde(default)]
    pub book_side: Option<BookSide>,
    #[serde(borrow)]
    pub count_fp: Cow<'a, str>,
    pub yes_price_dollars: Cow<'a, str>,
    pub is_taker: bool,
    #[serde(borrow)]
    pub fee_cost: Cow<'a, str>,
    pub ts: i64,
    pub ts_ms: i64,
    #[serde(borrow)]
    pub post_position_fp: Cow<'a, str>,
    pub purchased_side: YesNo,
    #[serde(default, borrow)]
    pub created_time: Option<Cow<'a, str>>,
    #[serde(default)]
    #[serde(alias = "subaccount_number")]
    pub subaccount: Option<i64>,
}

impl<'a> WsFillRef<'a> {
    pub fn into_owned(self) -> WsFill {
        WsFill {
            trade_id: self.trade_id.into_owned(),
            order_id: self.order_id.into_owned(),
            client_order_id: self.client_order_id.map(Cow::into_owned),
            market_ticker: self.market_ticker.into_owned(),
            exchange_index: self.exchange_index,
            side: self.side,
            action: self.action,
            outcome_side: self.outcome_side,
            book_side: self.book_side,
            count_fp: self.count_fp.into_owned(),
            yes_price_dollars: self.yes_price_dollars.into_owned(),
            is_taker: self.is_taker,
            fee_cost: self.fee_cost.into_owned(),
            ts: self.ts,
            ts_ms: self.ts_ms,
            post_position_fp: self.post_position_fp.into_owned(),
            purchased_side: self.purchased_side,
            created_time: self.created_time.map(Cow::into_owned),
            subaccount: self.subaccount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_fill_legacy_side_action_parse() {
        let json = r#"{
            "trade_id":"t",
            "order_id":"o",
            "market_ticker":"T",
            "side":"no",
            "action":"buy",
            "count_fp":"1",
            "yes_price_dollars":"0.01",
            "is_taker":true,
            "fee_cost":"0.00",
            "ts":0,
            "ts_ms":0,
            "post_position_fp":"1.00",
            "purchased_side":"yes"
        }"#;
        let fill: WsFill = serde_json::from_str(json).unwrap();
        assert!(matches!(fill.side, Some(YesNo::No)));
        assert!(matches!(fill.action, Some(BuySell::Buy)));
    }

    #[test]
    fn ws_fill_normalized_fields_parse() {
        let json = r#"{
            "trade_id":"t",
            "order_id":"o",
            "market_ticker":"T",
            "outcome_side":"yes",
            "book_side":"bid",
            "count_fp":"1",
            "yes_price_dollars":"0.01",
            "is_taker":true,
            "fee_cost":"0.00",
            "ts":0,
            "ts_ms":0,
            "post_position_fp":"1.00",
            "purchased_side":"yes"
        }"#;
        let fill: WsFill = serde_json::from_str(json).unwrap();
        assert!(matches!(fill.outcome_side, Some(YesNo::Yes)));
        assert!(matches!(fill.book_side, Some(BookSide::Bid)));
        assert!(fill.side.is_none());
        assert!(fill.action.is_none());
    }
}
