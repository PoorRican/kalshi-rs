use serde::{Deserialize, Serialize};

use super::FeeType;

#[derive(Debug, Clone, Deserialize)]
pub struct GetExchangeStatusResponse {
    pub exchange_active: bool,
    pub trading_active: bool,
    #[serde(default)]
    pub exchange_estimated_resume_time: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementType {
    Info,
    Warning,
    Error,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementStatus {
    Active,
    Inactive,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Announcement {
    #[serde(rename = "type")]
    pub r#type: AnnouncementType,
    pub message: String,
    pub delivery_time: String,
    pub status: AnnouncementStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetExchangeAnnouncementsResponse {
    pub announcements: Vec<Announcement>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DailySchedule {
    pub open_time: String,
    pub close_time: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StandardHours {
    pub start_time: String,
    pub end_time: String,
    #[serde(default)]
    pub monday: Vec<DailySchedule>,
    #[serde(default)]
    pub tuesday: Vec<DailySchedule>,
    #[serde(default)]
    pub wednesday: Vec<DailySchedule>,
    #[serde(default)]
    pub thursday: Vec<DailySchedule>,
    #[serde(default)]
    pub friday: Vec<DailySchedule>,
    #[serde(default)]
    pub saturday: Vec<DailySchedule>,
    #[serde(default)]
    pub sunday: Vec<DailySchedule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaintenanceWindow {
    pub start_datetime: String,
    pub end_datetime: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeSchedule {
    #[serde(default)]
    pub standard_hours: Vec<StandardHours>,
    #[serde(default)]
    pub maintenance_windows: Vec<MaintenanceWindow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetExchangeScheduleResponse {
    pub schedule: ExchangeSchedule,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUserDataTimestampResponse {
    pub as_of_time: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeriesFeeChange {
    pub id: i64,
    pub series_ticker: String,
    pub fee_type: FeeType,
    pub fee_multiplier: i64,
    pub scheduled_ts: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetSeriesFeeChangesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_historical: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesFeeChangesResponse {
    #[serde(rename = "series_fee_change_arr")]
    pub series_fee_change_arr: Vec<SeriesFeeChange>,
}
