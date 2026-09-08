//! Portfolio endpoints: balance, positions, fills, settlements, resting order value.
//!
//! All endpoints here require authentication. Positions and orders share the
//! same taxonomy split (market vs event), and fills/settlements provide
//! post-trade accounting feeds.

use crate::KalshiError;
use crate::rest::account::EmptyResponse;
use crate::rest::client::KalshiRestClient;
use crate::rest::pagination::{CursorPager, stream_items};
use crate::types::{
    BookSide, BuySell, FixedPointCount, FixedPointDollars, PositionCountFilter, YesNo,
    deserialize_null_as_empty_vec, serialize_csv_opt,
};
use futures::stream::Stream;
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetBalanceResponse {
    pub balance: i64,
    pub portfolio_value: i64,
    pub updated_ts: i64,
    /// Centi-cent precision dollar balance (direct members only). Added 2026-05-28.
    #[serde(default)]
    pub balance_dollars: Option<FixedPointDollars>,
}

/// GET /portfolio/balance query params.
///
/// Both `balance` and `portfolio_value` cover all exchange indexes unless
/// `exchange_index` is provided. Added 2026 (exchange sharding rollout).
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetBalanceParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<i64>,
}

/// GET /portfolio/positions query params
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetPositionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 100, max 1000

    /// CSV of non-zero filters (position,total_traded)
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_csv_opt"
    )]
    pub count_filter: Option<Vec<PositionCountFilter>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// CSV max 10
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_csv_opt"
    )]
    pub event_ticker: Option<Vec<String>>,

    /// 0..=32
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,

    /// Filter to a single exchange shard. Omit to return results from all shards.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<i64>,
}

impl GetPositionsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit
            && (limit == 0 || limit > 1000)
        {
            return Err(KalshiError::InvalidParams(
                "GET /portfolio/positions: limit must be 1..=1000".to_string(),
            ));
        }
        if let Some(evts) = &self.event_ticker
            && evts.len() > 10
        {
            return Err(KalshiError::InvalidParams(
                "GET /portfolio/positions: event_ticker supports up to 10 tickers".to_string(),
            ));
        }
        if let Some(sub) = self.subaccount
            && sub > 32
        {
            return Err(KalshiError::InvalidParams(
                "subaccount must be 0..=32".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketPosition {
    pub ticker: String,
    /// Exchange shard this position lives on. Added 2026 (exchange sharding rollout).
    pub exchange_index: i64,
    pub total_traded_dollars: FixedPointDollars,
    pub position_fp: FixedPointCount,
    pub market_exposure_dollars: FixedPointDollars,
    pub realized_pnl_dollars: FixedPointDollars,
    pub fees_paid_dollars: FixedPointDollars,
    pub last_updated_ts: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventPosition {
    pub event_ticker: String,
    pub total_cost_dollars: FixedPointDollars,
    pub total_cost_shares_fp: FixedPointCount,
    pub event_exposure_dollars: FixedPointDollars,
    pub realized_pnl_dollars: FixedPointDollars,
    pub fees_paid_dollars: FixedPointDollars,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetPositionsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub market_positions: Vec<MarketPosition>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub event_positions: Vec<EventPosition>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PositionsPage {
    pub market_positions: Vec<MarketPosition>,
    pub event_positions: Vec<EventPosition>,
}

impl From<GetPositionsResponse> for PositionsPage {
    fn from(resp: GetPositionsResponse) -> Self {
        Self {
            market_positions: resp.market_positions,
            event_positions: resp.event_positions,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settlement {
    pub ticker: String,
    /// Exchange shard this settlement occurred on. Added 2026 (exchange sharding rollout).
    pub exchange_index: i64,
    pub event_ticker: String,
    pub market_result: String,
    pub yes_count_fp: FixedPointCount,
    pub yes_total_cost_dollars: FixedPointDollars,
    pub no_count_fp: FixedPointCount,
    pub no_total_cost_dollars: FixedPointDollars,
    pub revenue: i64,
    pub settled_time: String,
    pub fee_cost: FixedPointDollars,
    #[serde(default)]
    pub value: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetSettlementsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetSettlementsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub settlements: Vec<Settlement>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fill {
    pub fill_id: String,
    /// Exchange shard this fill occurred on. Added 2026 (exchange sharding rollout).
    pub exchange_index: i64,
    pub order_id: String,
    pub trade_id: String,
    pub ticker: String,
    pub market_ticker: String,
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
    pub count_fp: FixedPointCount,
    pub yes_price_dollars: FixedPointDollars,
    pub no_price_dollars: FixedPointDollars,
    pub is_taker: bool,
    pub fee_cost: FixedPointDollars,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub subaccount_number: Option<u32>,
    #[serde(default)]
    pub ts: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetFillsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    /// Filter to a single exchange shard. Omit to return results from all shards.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetFillsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub fills: Vec<Fill>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Total resting order value for one exchange shard.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IndexedBalance {
    pub exchange_index: i64,
    pub balance: FixedPointDollars,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetPortfolioRestingOrderTotalValueResponse {
    pub total_resting_order_value: i64,
    /// Per-exchange-index breakdown of resting order value. Added 2026 (exchange sharding rollout).
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub resting_order_value_breakdown: Vec<IndexedBalance>,
}

/// `POST`/`GET /portfolio/target_balance_allocation` types.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TargetBalanceAllocation {
    pub exchange_index: i64,
    pub percent: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetTargetBalanceAllocationResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub allocations: Vec<TargetBalanceAllocation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetTargetBalanceAllocationRequest {
    pub allocations: Vec<TargetBalanceAllocation>,
    /// Collateral an automatic rebalance leaves behind for resting orders
    /// (`"max"` or `"sum"`). Defaults to `"sum"` when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resting_margin_reservation: Option<String>,
}

/// `POST /portfolio/intra_exchange_instance_transfer` request body.
#[derive(Debug, Clone, Serialize)]
pub struct IntraExchangeInstanceTransferRequest {
    /// `"event_contract"` or `"margined"`.
    pub source: String,
    pub destination: String,
    /// Amount to transfer, in centicents.
    pub amount: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_exchange_shard: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_exchange_shard: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntraExchangeInstanceTransferResponse {
    pub transfer_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntraExchangeInstanceTransfer {
    pub transfer_id: String,
    pub source: String,
    pub destination: String,
    pub source_exchange_shard: i64,
    pub destination_exchange_shard: i64,
    /// Transfer amount in dollars.
    pub amount: FixedPointDollars,
    /// `"pending"` or `"complete"`.
    pub status: String,
    pub created_ts: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetIntraExchangeInstanceTransfersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetIntraExchangeInstanceTransfersResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub transfers: Vec<IntraExchangeInstanceTransfer>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetIntraExchangeInstanceTransferResponse {
    pub transfer: IntraExchangeInstanceTransfer,
}

impl KalshiRestClient {
    /// Get the account balance.
    ///
    /// **Requires auth.**
    pub async fn get_balance(&self) -> Result<GetBalanceResponse, KalshiError> {
        self.get_balance_scoped(GetBalanceParams::default()).await
    }

    /// Get the account balance, optionally scoped to a subaccount and/or exchange index.
    ///
    /// **Requires auth.**
    pub async fn get_balance_scoped(
        &self,
        params: GetBalanceParams,
    ) -> Result<GetBalanceResponse, KalshiError> {
        let path = Self::full_path("/portfolio/balance");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// Transfer funds within the same account across exchange instances/shards.
    ///
    /// **Requires auth.**
    pub async fn intra_exchange_instance_transfer(
        &self,
        body: IntraExchangeInstanceTransferRequest,
    ) -> Result<IntraExchangeInstanceTransferResponse, KalshiError> {
        let path = Self::full_path("/portfolio/intra_exchange_instance_transfer");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// List intra-exchange-instance transfer history. Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_intra_exchange_instance_transfers(
        &self,
        params: GetIntraExchangeInstanceTransfersParams,
    ) -> Result<GetIntraExchangeInstanceTransfersResponse, KalshiError> {
        let path = Self::full_path("/portfolio/intra_exchange_instance_transfers");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// Get a single intra-exchange-instance transfer by id.
    ///
    /// **Requires auth.**
    pub async fn get_intra_exchange_instance_transfer(
        &self,
        transfer_id: &str,
    ) -> Result<GetIntraExchangeInstanceTransferResponse, KalshiError> {
        let path = Self::full_path(&format!(
            "/portfolio/intra_exchange_instance_transfers/{transfer_id}"
        ));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// Get the caller's target balance allocation across exchange indexes.
    ///
    /// **Requires auth.**
    pub async fn get_target_balance_allocation(
        &self,
    ) -> Result<GetTargetBalanceAllocationResponse, KalshiError> {
        let path = Self::full_path("/portfolio/target_balance_allocation");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// Replace the caller's target balance allocation across exchange indexes.
    /// Percentages must total 100; an empty `allocations` array disables automatic rebalancing.
    ///
    /// **Requires auth.**
    pub async fn set_target_balance_allocation(
        &self,
        body: SetTargetBalanceAllocationRequest,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path("/portfolio/target_balance_allocation");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// Create a pager for iterating over intra-exchange-instance transfers page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn intra_exchange_instance_transfers_pager(
        &self,
        params: GetIntraExchangeInstanceTransfersParams,
    ) -> CursorPager<IntraExchangeInstanceTransfer> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client
                    .get_intra_exchange_instance_transfers(page_params)
                    .await?;
                Ok((resp.transfers, resp.cursor))
            })
        })
    }

    /// Stream intra-exchange-instance transfers one by one.
    ///
    /// **Requires auth.**
    pub fn stream_intra_exchange_instance_transfers(
        &self,
        params: GetIntraExchangeInstanceTransfersParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<IntraExchangeInstanceTransfer, KalshiError>> + Send {
        stream_items(
            self.intra_exchange_instance_transfers_pager(params),
            max_items,
        )
    }

    /// List open positions. Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_positions(
        &self,
        params: GetPositionsParams,
    ) -> Result<GetPositionsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/portfolio/positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// List settled positions archived to the historical database. Positions whose
    /// markets were archived before `market_positions_last_updated_ts` on
    /// `GET /historical/cutoff` are available here; unsettled positions remain on
    /// [`Self::get_positions`]. Positions are archived per whole event and never
    /// split between the two endpoints. Added 2026-07-23.
    ///
    /// **Requires auth.**
    pub async fn get_historical_positions(
        &self,
        params: GetPositionsParams,
    ) -> Result<GetPositionsResponse, KalshiError> {
        let path = Self::full_path("/historical/positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// List fills (executed trades). Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_fills(&self, params: GetFillsParams) -> Result<GetFillsResponse, KalshiError> {
        let path = Self::full_path("/portfolio/fills");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// List settlements. Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_settlements(
        &self,
        params: GetSettlementsParams,
    ) -> Result<GetSettlementsResponse, KalshiError> {
        let path = Self::full_path("/portfolio/settlements");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn get_portfolio_total_resting_order_value(
        &self,
    ) -> Result<GetPortfolioRestingOrderTotalValueResponse, KalshiError> {
        let path = Self::full_path("/portfolio/summary/total_resting_order_value");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// Create a pager for iterating over positions page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn positions_pager(&self, params: GetPositionsParams) -> CursorPager<PositionsPage> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_positions(page_params).await?;
                let cursor = resp.cursor.clone();
                let page = PositionsPage::from(resp);
                Ok((vec![page], cursor))
            })
        })
    }

    /// Create a pager for iterating over fills page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn fills_pager(&self, params: GetFillsParams) -> CursorPager<Fill> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_fills(page_params).await?;
                Ok((resp.fills, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over settlements page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn settlements_pager(&self, params: GetSettlementsParams) -> CursorPager<Settlement> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_settlements(page_params).await?;
                Ok((resp.settlements, resp.cursor))
            })
        })
    }

    /// Stream positions one by one.
    ///
    /// **Requires auth.**
    pub fn stream_positions(
        &self,
        params: GetPositionsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<PositionsPage, KalshiError>> + Send {
        stream_items(self.positions_pager(params), max_items)
    }

    /// Stream fills one by one.
    ///
    /// **Requires auth.**
    pub fn stream_fills(
        &self,
        params: GetFillsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Fill, KalshiError>> + Send {
        stream_items(self.fills_pager(params), max_items)
    }

    /// Stream settlements one by one.
    ///
    /// **Requires auth.**
    pub fn stream_settlements(
        &self,
        params: GetSettlementsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Settlement, KalshiError>> + Send {
        stream_items(self.settlements_pager(params), max_items)
    }
}
