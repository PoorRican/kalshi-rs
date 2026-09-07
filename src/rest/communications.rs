//! RFQ (Request For Quote) and Quote endpoints, plus the communications ID.
//!
//! RFQs express interest in a given market and notional. Market makers respond
//! with quotes that the RFQ creator can accept/confirm to execute a trade.
//! All endpoints require authentication.

use crate::KalshiError;
use crate::rest::account::EmptyResponse;
use crate::rest::client::KalshiRestClient;
use crate::rest::markets::MveSelectedLeg;
use crate::rest::pagination::{CursorPager, stream_items};
use crate::types::{FixedPointCount, FixedPointDollars, YesNo, deserialize_null_as_empty_vec};
use futures::stream::Stream;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetCommunicationsIdResponse {
    pub communications_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Quote {
    pub id: String,
    pub rfq_id: String,
    pub creator_id: String,
    pub rfq_creator_id: String,
    pub market_ticker: String,
    pub contracts_fp: FixedPointCount,
    pub yes_bid_dollars: FixedPointDollars,
    pub no_bid_dollars: FixedPointDollars,
    pub created_ts: String,
    pub updated_ts: String,
    pub status: String,
    #[serde(default)]
    pub accepted_side: Option<YesNo>,
    #[serde(default)]
    pub accepted_ts: Option<String>,
    #[serde(default)]
    pub confirmed_ts: Option<String>,
    #[serde(default)]
    pub executed_ts: Option<String>,
    #[serde(default)]
    pub cancelled_ts: Option<String>,
    #[serde(default)]
    pub rest_remainder: Option<bool>,
    #[serde(default)]
    pub cancellation_reason: Option<String>,
    #[serde(default)]
    pub creator_user_id: Option<String>,
    #[serde(default)]
    pub rfq_creator_user_id: Option<String>,
    #[serde(default)]
    pub rfq_target_cost_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub rfq_creator_order_id: Option<String>,
    #[serde(default)]
    pub creator_order_id: Option<String>,
    #[serde(default)]
    pub yes_contracts_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub no_contracts_fp: Option<FixedPointCount>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RFQ {
    pub id: String,
    pub creator_id: String,
    pub market_ticker: String,
    pub contracts_fp: FixedPointCount,
    #[serde(default)]
    pub target_cost_dollars: Option<FixedPointDollars>,
    pub status: String,
    pub created_ts: String,
    #[serde(default)]
    pub mve_collection_ticker: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub mve_selected_legs: Vec<MveSelectedLeg>,
    #[serde(default)]
    pub rest_remainder: Option<bool>,
    #[serde(default)]
    pub cancellation_reason: Option<String>,
    #[serde(default)]
    pub creator_user_id: Option<String>,
    #[serde(default)]
    pub cancelled_ts: Option<String>,
    #[serde(default)]
    pub updated_ts: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetQuotesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Restrict to quotes last updated after this Unix timestamp. Added 2026-06-18.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    /// Restrict to quotes last updated before this Unix timestamp. Added 2026-06-18.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_creator_user_id: Option<String>,
    /// Filter to quotes created by the authenticated user. Pass `"self"` to enable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_creator_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_creator_subtrader_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_id: Option<String>,
    /// Filter to quotes responding to RFQs created by the authenticated user.
    /// Pass `"self"` to enable. Added 2026-05-07.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_user_filter: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetQuotesResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub quotes: Vec<Quote>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetQuoteResponse {
    pub quote: Quote,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateQuoteRequest {
    pub rfq_id: String,
    pub yes_bid: String,
    pub no_bid: String,
    pub rest_remainder: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateQuoteResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AcceptQuoteRequest {
    pub accepted_side: YesNo,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetRFQsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_user_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetRFQsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub rfqs: Vec<RFQ>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetRFQResponse {
    pub rfq: RFQ,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateRFQRequest {
    pub market_ticker: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_fp: Option<FixedPointCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cost_centi_cents: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cost_dollars: Option<FixedPointDollars>,
    pub rest_remainder: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_existing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtrader_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRFQResponse {
    pub id: String,
}

impl KalshiRestClient {
    pub async fn get_communications_id(&self) -> Result<GetCommunicationsIdResponse, KalshiError> {
        let path = Self::full_path("/communications/id");
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn get_rfqs(&self, params: GetRFQsParams) -> Result<GetRFQsResponse, KalshiError> {
        let path = Self::full_path("/communications/rfqs");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn create_rfq(
        &self,
        body: CreateRFQRequest,
    ) -> Result<CreateRFQResponse, KalshiError> {
        let path = Self::full_path("/communications/rfqs");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_rfq(&self, rfq_id: &str) -> Result<GetRFQResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/rfqs/{rfq_id}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn delete_rfq(&self, rfq_id: &str) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/rfqs/{rfq_id}"));
        self.send(
            Method::DELETE,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn get_quotes(
        &self,
        params: GetQuotesParams,
    ) -> Result<GetQuotesResponse, KalshiError> {
        let path = Self::full_path("/communications/quotes");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn create_quote(
        &self,
        body: CreateQuoteRequest,
    ) -> Result<CreateQuoteResponse, KalshiError> {
        let path = Self::full_path("/communications/quotes");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_quote(&self, quote_id: &str) -> Result<GetQuoteResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/quotes/{quote_id}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn delete_quote(&self, quote_id: &str) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/quotes/{quote_id}"));
        self.send(
            Method::DELETE,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn accept_quote(
        &self,
        quote_id: &str,
        body: AcceptQuoteRequest,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/quotes/{quote_id}/accept"));
        self.send(Method::PUT, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn confirm_quote(&self, quote_id: &str) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/communications/quotes/{quote_id}/confirm"));
        let body = EmptyResponse::default();
        self.send(Method::PUT, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// Create a pager for iterating over RFQs page by page.
    pub fn rfqs_pager(&self, params: GetRFQsParams) -> CursorPager<RFQ> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_rfqs(page_params).await?;
                Ok((resp.rfqs, resp.cursor))
            })
        })
    }

    /// Create a pager for iterating over quotes page by page.
    pub fn quotes_pager(&self, params: GetQuotesParams) -> CursorPager<Quote> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_quotes(page_params).await?;
                Ok((resp.quotes, resp.cursor))
            })
        })
    }

    /// Stream RFQs one by one.
    pub fn stream_rfqs(
        &self,
        params: GetRFQsParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<RFQ, KalshiError>> + Send {
        stream_items(self.rfqs_pager(params), max_items)
    }

    /// Stream quotes one by one.
    pub fn stream_quotes(
        &self,
        params: GetQuotesParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Quote, KalshiError>> + Send {
        stream_items(self.quotes_pager(params), max_items)
    }

    /// Fetch all pages for RFQs using cursor pagination.
    pub async fn get_rfqs_all(&self, params: GetRFQsParams) -> Result<Vec<RFQ>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_rfqs(page_params).await?;
                Ok((resp.rfqs, resp.cursor))
            }
        })
        .await
    }

    /// Fetch all pages for quotes using cursor pagination.
    pub async fn get_quotes_all(&self, params: GetQuotesParams) -> Result<Vec<Quote>, KalshiError> {
        self.paginate_cursor(params.cursor.clone(), |cursor| {
            let mut page_params = params.clone();
            page_params.cursor = cursor;
            async move {
                let resp = self.get_quotes(page_params).await?;
                Ok((resp.quotes, resp.cursor))
            }
        })
        .await
    }
}
