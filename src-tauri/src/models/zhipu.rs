use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhipuUsageInfo {
    pub token_total: i64,
    pub token_used: i64,
    pub token_remaining: i64,
    pub token_percentage: f64,
    pub token_hours: i64,
    pub token_reset_time: String,
    pub mcp_total: i64,
    pub mcp_used: i64,
    pub mcp_remaining: i64,
    pub mcp_percentage: i64,
    pub mcp_reset_time: String,
    pub mcp_search: i64,
    pub mcp_web: i64,
    pub mcp_zread: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZhipuApiResponse {
    pub code: i32,
    pub data: Option<ZhipuQuotaData>,
    pub msg: Option<String>,
    pub success: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZhipuQuotaData {
    pub limits: Vec<ZhipuLimit>,
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZhipuLimit {
    #[serde(rename = "type")]
    pub limit_type: String,
    pub unit: i64,
    pub number: i64,
    pub usage: Option<i64>,
    pub percentage: i64,
    pub remaining: Option<i64>,
    #[serde(rename = "nextResetTime")]
    pub next_reset_time: i64,
    #[serde(rename = "usageDetails")]
    pub usage_details: Option<Vec<ZhipuUsageDetail>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZhipuUsageDetail {
    #[serde(rename = "modelCode")]
    pub model_code: String,
    pub usage: i64,
}
