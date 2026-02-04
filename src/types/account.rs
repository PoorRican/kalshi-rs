use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GetAccountApiLimitsResponse {
    pub usage_tier: String,
    pub read_limit: i64,
    pub write_limit: i64,
}
