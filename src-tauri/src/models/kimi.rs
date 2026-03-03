use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KimiUsageInfo {
    pub hourly_total: i64,
    pub hourly_used: i64,
    pub hourly_remaining: i64,
    pub hourly_percentage: f64,
    pub hourly_reset_time: String,
    pub hourly_window: i64,
    pub weekly_total: i64,
    pub weekly_used: i64,
    pub weekly_remaining: i64,
    pub weekly_percentage: f64,
    pub weekly_reset_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KimiApiResponse {
    pub usages: Vec<KimiUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KimiUsage {
    pub scope: String,
    pub detail: KimiUsageDetail,
    pub limits: Vec<KimiUsageLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KimiUsageDetail {
    #[serde(rename = "limit")]
    pub limit_value: String,
    pub used: String,
    pub remaining: String,
    #[serde(rename = "resetTime")]
    pub reset_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KimiUsageLimit {
    pub window: KimiWindow,
    pub detail: KimiUsageLimitDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KimiWindow {
    pub duration: i64,
    #[serde(rename = "timeUnit")]
    pub time_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KimiUsageLimitDetail {
    #[serde(rename = "limit")]
    pub limit_value: String,
    pub remaining: String,
    #[serde(rename = "resetTime")]
    pub reset_time: String,
}
