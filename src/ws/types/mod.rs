use crate::rest::{EventPosition, MarketPosition};
use serde::Deserialize;
use std::borrow::Cow;

mod channel;
pub use channel::*;

mod msg_type;
pub use msg_type::*;

pub mod messages;
pub use messages::*;

mod subscription;
pub use subscription::*;

mod commands;
pub(crate) use commands::*;

mod wire;

mod envelope;
pub use envelope::*;

#[derive(Debug, Clone, Deserialize)]
pub struct MarketPositionRef<'a> {
    #[serde(borrow)]
    pub ticker: Cow<'a, str>,
    #[serde(default)]
    pub exchange_index: Option<u32>,
    #[serde(borrow)]
    pub total_traded_dollars: FixedPointDollarsRef<'a>,
    #[serde(borrow)]
    pub position_fp: FixedPointCountRef<'a>,
    #[serde(borrow)]
    pub market_exposure_dollars: FixedPointDollarsRef<'a>,
    #[serde(borrow)]
    pub realized_pnl_dollars: FixedPointDollarsRef<'a>,
    #[serde(borrow)]
    pub fees_paid_dollars: FixedPointDollarsRef<'a>,
    #[serde(borrow)]
    pub last_updated_ts: Cow<'a, str>,
}

impl<'a> MarketPositionRef<'a> {
    pub fn into_owned(self) -> MarketPosition {
        MarketPosition {
            ticker: self.ticker.into_owned(),
            exchange_index: self.exchange_index,
            total_traded_dollars: self.total_traded_dollars.into_owned(),
            position_fp: self.position_fp.into_owned(),
            market_exposure_dollars: self.market_exposure_dollars.into_owned(),
            realized_pnl_dollars: self.realized_pnl_dollars.into_owned(),
            fees_paid_dollars: self.fees_paid_dollars.into_owned(),
            last_updated_ts: self.last_updated_ts.into_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventPositionRef<'a> {
    #[serde(borrow)]
    pub event_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub total_cost_dollars: FixedPointDollarsRef<'a>,
    #[serde(borrow)]
    pub total_cost_shares_fp: FixedPointCountRef<'a>,
    #[serde(borrow)]
    pub event_exposure_dollars: FixedPointDollarsRef<'a>,
    #[serde(borrow)]
    pub realized_pnl_dollars: FixedPointDollarsRef<'a>,
    #[serde(borrow)]
    pub fees_paid_dollars: FixedPointDollarsRef<'a>,
}

impl<'a> EventPositionRef<'a> {
    pub fn into_owned(self) -> EventPosition {
        EventPosition {
            event_ticker: self.event_ticker.into_owned(),
            total_cost_dollars: self.total_cost_dollars.into_owned(),
            total_cost_shares_fp: self.total_cost_shares_fp.into_owned(),
            event_exposure_dollars: self.event_exposure_dollars.into_owned(),
            realized_pnl_dollars: self.realized_pnl_dollars.into_owned(),
            fees_paid_dollars: self.fees_paid_dollars.into_owned(),
        }
    }
}
