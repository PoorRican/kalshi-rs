use crate::types::{BookSide, TradeTakerSide};
use serde::Deserialize;
use std::borrow::Cow;

/// Trade channel message (type: "trade")
#[derive(Debug, Clone, Deserialize)]
pub struct WsTrade {
    pub trade_id: String,
    #[serde(alias = "ticker")]
    pub market_ticker: String,
    pub count_fp: String,
    pub yes_price_dollars: String,
    pub no_price_dollars: String,
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
    /// True if the trade was matched off book as a block trade. Added 2026-08-13.
    #[serde(default)]
    pub is_block_trade: Option<bool>,
    pub ts: i64,
    /// Spec marks `ts_ms` as required, but the exchange occasionally omits it.
    /// See `docs/spec-parity.md`.
    #[serde(default)]
    pub ts_ms: Option<i64>,
    #[serde(default)]
    pub created_time: Option<String>,
}

/// Trade channel message (type: "trade")
#[derive(Debug, Clone, Deserialize)]
pub struct WsTradeRef<'a> {
    #[serde(borrow)]
    pub trade_id: Cow<'a, str>,
    #[serde(alias = "ticker", borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub count_fp: Cow<'a, str>,
    #[serde(borrow)]
    pub yes_price_dollars: Cow<'a, str>,
    #[serde(borrow)]
    pub no_price_dollars: Cow<'a, str>,
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
    /// True if the trade was matched off book as a block trade. Added 2026-08-13.
    #[serde(default)]
    pub is_block_trade: Option<bool>,
    pub ts: i64,
    /// Spec marks `ts_ms` as required, but the exchange occasionally omits it.
    /// See `docs/spec-parity.md`.
    #[serde(default)]
    pub ts_ms: Option<i64>,
    #[serde(default, borrow)]
    pub created_time: Option<Cow<'a, str>>,
}

impl<'a> WsTradeRef<'a> {
    pub fn into_owned(self) -> WsTrade {
        WsTrade {
            trade_id: self.trade_id.into_owned(),
            market_ticker: self.market_ticker.into_owned(),
            count_fp: self.count_fp.into_owned(),
            yes_price_dollars: self.yes_price_dollars.into_owned(),
            no_price_dollars: self.no_price_dollars.into_owned(),
            taker_side: self.taker_side,
            taker_outcome_side: self.taker_outcome_side,
            taker_book_side: self.taker_book_side,
            is_block_trade: self.is_block_trade,
            ts: self.ts,
            ts_ms: self.ts_ms,
            created_time: self.created_time.map(Cow::into_owned),
        }
    }
}
