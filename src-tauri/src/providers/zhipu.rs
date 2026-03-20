//! 智谱 API 返回的额度数据示例：
//! ```json
//! {
//!     "code": 200,
//!     "msg": "操作成功",
//!     "data": {
//!         "limits": [
//!             {
//!                 "type": "TIME_LIMIT",      // MCP 额度
//!                 "unit": 5,                  // 5=月
//!                 "number": 1,                // 每月
//!                 "usage": 1000,             // 总额度 1000 次
//!                 "currentValue": 12,        // 使用额度 12 次
//!                 "remaining": 988,          // 剩余额度
//!                 "percentage": 1,           // 使用额度 1%
//!                 "nextResetTime": 1776397506998,
//!                 "usageDetails": [
//!                     { "modelCode": "search-prime", "usage": 12 },
//!                     { "modelCode": "web-reader", "usage": 0 },
//!                     { "modelCode": "zread", "usage": 0 }
//!                 ]
//!             },
//!             {
//!                 "type": "TOKENS_LIMIT",     // 小时 token 额度
//!                 "unit": 3,                  // 3=小时
//!                 "number": 5,                 // 每 5 小时
//!                 "percentage": 5,             // 使用额度 5%
//!                 "nextResetTime": 1773920038794
//!             },
//!             {
//!                 "type": "TOKENS_LIMIT",     // 周 token 额度
//!                 "unit": 6,                  // 6=周
//!                 "number": 1,                 // 每周
//!                 "percentage": 32,            // 使用额度 32%
//!                 "nextResetTime": 1774323906997
//!             }
//!         ],
//!         "level": "pro"
//!     },
//!     "success": true
//! }
//! ```

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::future::Future;

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, IsMenuItem, MenuItem, PredefinedMenuItem},
    AppHandle, Wry,
};

use crate::error::{AppError, Result};
use crate::provider::{format_progress_bar, format_timestamp_js, Provider, ProviderRegistry, UsageInfo};

pub const ZHIPU_ID: &str = "zhipu-coding-plan";
pub const ZHIPU_DISPLAY_NAME: &str = "智谱";
pub const ZHIPU_LOGIN_ARG: &str = "zhipu";
pub const ZHIPU_AUTH_TOKEN: &str = "bigmodel_token_production";

/// 时间单位枚举，对应 API 返回的 unit 字段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Hours = 3,
    Weeks = 6,
    Month = 5,
}

/// 分隔符长度
const SEPARATOR_LENGTH: usize = 25;

pub struct ZhipuProvider;

impl Provider for ZhipuProvider {
    fn id(&self) -> &'static str { ZHIPU_ID }
    fn display_name(&self) -> &'static str { ZHIPU_DISPLAY_NAME }
    fn login_script_arg(&self) -> &'static str { ZHIPU_LOGIN_ARG }
    fn auth_token_name(&self) -> &'static str { ZHIPU_AUTH_TOKEN }
    
    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<Box<dyn UsageInfo>>> + Send + 'static>> {
        Box::pin(async move {
            let cookies = read_cookies(&cookie_path)?;
            let token = find_cookie_value(&cookies, ZHIPU_AUTH_TOKEN)
                .ok_or_else(|| AppError::Auth("未找到认证 token".to_string()))?;
            
            let client = reqwest::Client::builder()
                .build()
                .map_err(|e| AppError::Http(e.to_string()))?;
            
            let response = client
                .get("https://bigmodel.cn/api/monitor/usage/quota/limit")
                .header("authorization", token)
                .header("referer", "https://bigmodel.cn/usercenter/glm-coding/usage")
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                .send()
                .await
                .map_err(|e| AppError::Http(e.to_string()))?;
            
            let response_text = response
                .text()
                .await
                .map_err(|e| AppError::Http(e.to_string()))?;
            
            let api_response: ZhipuApiResponse = serde_json::from_str(&response_text)
                .map_err(|e| AppError::Parse(format!("{} - {}", e, response_text)))?;
            
            if api_response.code != 200 {
                return Err(AppError::Http(api_response.msg.unwrap_or("获取数据失败".to_string())));
            }
            
            let data = api_response.data.ok_or(AppError::Parse("响应数据为空".to_string()))?;

            let time_limit = data.limits.iter().find(|l| l.limit_type == "TIME_LIMIT");
            // 查找小时 token 额度
            let hourly_token_limit = data
                .limits
                .iter()
                .find(|l| l.limit_type == "TOKENS_LIMIT" && l.unit == TimeUnit::Hours as i64);
            // 查找周 token 额度
            let weekly_token_limit = data
                .limits
                .iter()
                .find(|l| l.limit_type == "TOKENS_LIMIT" && l.unit == TimeUnit::Weeks as i64);

            let info = build_usage_info(hourly_token_limit, weekly_token_limit, time_limit);
            Ok(Box::new(info) as Box<dyn UsageInfo>)
        })
    }
}

pub static ZHIPU: ZhipuProvider = ZhipuProvider;

inventory::submit!(ProviderRegistry(&ZHIPU));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZhipuUsageInfo {
    // 小时 Token 额度
    pub token_percentage: f64,
    pub token_period: String,
    pub token_reset_time: String,

    // 周 Token 额度
    pub weekly_token_percentage: f64,
    pub weekly_token_period: String,
    pub weekly_token_reset_time: String,

    // MCP 额度
    pub mcp_total: i64,
    pub mcp_used: i64,
    pub mcp_remaining: i64,
    pub mcp_percentage: i64,
    pub mcp_reset_time: String,
    pub mcp_search: i64,
    pub mcp_web: i64,
    pub mcp_zread: i64,
}

impl UsageInfo for ZhipuUsageInfo {
    fn provider_id(&self) -> &'static str {
        ZHIPU_ID
    }

    fn is_token_exhausted(&self) -> bool {
        self.token_percentage >= 100.0
    }

    fn render_menu_items(
        &self,
        app: &AppHandle,
        is_selected: bool,
        handles: &mut crate::state::MenuHandles,
    ) -> Vec<Box<dyn IsMenuItem<Wry>>> {
        let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();

        items.push(Box::new(CheckMenuItem::with_id(
            app,
            format!("select-{}", ZHIPU_ID),
            format!("{} Coding Plan", ZHIPU_DISPLAY_NAME),
            true,
            is_selected,
            None::<&str>,
        ).unwrap()));

        // 小时额度显示
        let token_title = MenuItem::with_id(app, format!("{}-token-title", ZHIPU_ID), format!("Token 额度（{}）", self.token_period), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-token-title", ZHIPU_ID), token_title.clone());
        items.push(Box::new(token_title));

        let token_bar = MenuItem::with_id(app, format!("{}-token-bar", ZHIPU_ID), format_progress_bar(self.token_percentage), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-token-bar", ZHIPU_ID), token_bar.clone());
        items.push(Box::new(token_bar));

        let token_reset = MenuItem::with_id(app, format!("{}-token-reset", ZHIPU_ID), format!("重置: {}", self.token_reset_time), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-token-reset", ZHIPU_ID), token_reset.clone());
        items.push(Box::new(token_reset));

        items.push(Box::new(MenuItem::with_id(app, format!("{}-sep1", ZHIPU_ID), "-".repeat(SEPARATOR_LENGTH), false, None::<&str>).unwrap()));

        // 周额度显示
        let weekly_title = MenuItem::with_id(app, format!("{}-weekly-token-title", ZHIPU_ID), format!("Token 额度（{}）", self.weekly_token_period), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-weekly-token-title", ZHIPU_ID), weekly_title.clone());
        items.push(Box::new(weekly_title));

        let weekly_bar = MenuItem::with_id(app, format!("{}-weekly-token-bar", ZHIPU_ID), format_progress_bar(self.weekly_token_percentage), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-weekly-token-bar", ZHIPU_ID), weekly_bar.clone());
        items.push(Box::new(weekly_bar));

        let weekly_reset = MenuItem::with_id(app, format!("{}-weekly-token-reset", ZHIPU_ID), format!("重置: {}", self.weekly_token_reset_time), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-weekly-token-reset", ZHIPU_ID), weekly_reset.clone());
        items.push(Box::new(weekly_reset));

        items.push(Box::new(MenuItem::with_id(app, format!("{}-sep2", ZHIPU_ID), "-".repeat(SEPARATOR_LENGTH), false, None::<&str>).unwrap()));

        // MCP 额度
        let mcp_title = MenuItem::with_id(app, format!("{}-mcp-title", ZHIPU_ID), format!("MCP 额度（每月 {} 次）", self.mcp_total), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-mcp-title", ZHIPU_ID), mcp_title.clone());
        items.push(Box::new(mcp_title));

        let mcp_bar = MenuItem::with_id(app, format!("{}-mcp-bar", ZHIPU_ID), format_progress_bar(self.mcp_percentage as f64), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-mcp-bar", ZHIPU_ID), mcp_bar.clone());
        items.push(Box::new(mcp_bar));

        let mcp_detail = MenuItem::with_id(app, format!("{}-mcp-detail", ZHIPU_ID), format!("搜索: {} | 网页: {} | 阅读: {}", self.mcp_search, self.mcp_web, self.mcp_zread), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-mcp-detail", ZHIPU_ID), mcp_detail.clone());
        items.push(Box::new(mcp_detail));

        let mcp_reset = MenuItem::with_id(app, format!("{}-mcp-reset", ZHIPU_ID), format!("重置: {}", self.mcp_reset_time), false, None::<&str>).unwrap();
        handles.items.insert(format!("{}-mcp-reset", ZHIPU_ID), mcp_reset.clone());
        items.push(Box::new(mcp_reset));

        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));

        items
    }

    fn menu_item_updates(&self) -> Vec<(String, String)> {
        vec![
            (format!("{}-token-title", ZHIPU_ID), format!("Token 额度（{}）", self.token_period)),
            (format!("{}-token-bar", ZHIPU_ID), format_progress_bar(self.token_percentage)),
            (format!("{}-token-reset", ZHIPU_ID), format!("重置: {}", self.token_reset_time)),
            (format!("{}-weekly-token-title", ZHIPU_ID), format!("Token 额度（{}）", self.weekly_token_period)),
            (format!("{}-weekly-token-bar", ZHIPU_ID), format_progress_bar(self.weekly_token_percentage)),
            (format!("{}-weekly-token-reset", ZHIPU_ID), format!("重置: {}", self.weekly_token_reset_time)),
            (format!("{}-mcp-title", ZHIPU_ID), format!("MCP 额度（每月 {} 次）", self.mcp_total)),
            (format!("{}-mcp-bar", ZHIPU_ID), format_progress_bar(self.mcp_percentage as f64)),
            (format!("{}-mcp-detail", ZHIPU_ID), format!("搜索: {} | 网页: {} | 阅读: {}", self.mcp_search, self.mcp_web, self.mcp_zread)),
            (format!("{}-mcp-reset", ZHIPU_ID), format!("重置: {}", self.mcp_reset_time)),
        ]
    }

    fn clone_boxed(&self) -> Box<dyn UsageInfo> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ZhipuApiResponse {
    code: i32,
    data: Option<ZhipuQuotaData>,
    msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZhipuQuotaData {
    limits: Vec<ZhipuLimit>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZhipuLimit {
    #[serde(rename = "type")]
    limit_type: String,
    unit: i64,
    number: i64,
    usage: Option<i64>,
    percentage: i64,
    remaining: Option<i64>,
    #[serde(rename = "nextResetTime")]
    next_reset_time: Option<i64>,
    #[serde(rename = "usageDetails")]
    usage_details: Option<Vec<ZhipuUsageDetail>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZhipuUsageDetail {
    #[serde(rename = "modelCode")]
    model_code: String,
    usage: i64,
}

fn build_usage_info(
    hourly_token_limit: Option<&ZhipuLimit>,
    weekly_token_limit: Option<&ZhipuLimit>,
    time_limit: Option<&ZhipuLimit>,
) -> ZhipuUsageInfo {
    // 格式化时间周期显示
    fn format_period(unit: i64, number: i64) -> String {
        let unit_name = match unit {
            3 => "小时",
            6 => "周",
            _ => "未知",
        };
        if number == 1 {
            format!("每{}", unit_name)
        } else {
            format!("每 {} {}", number, unit_name)
        }
    }

    // 处理小时 token 额度
    let (token_percentage, token_period, token_reset_time) =
        if let Some(tl) = hourly_token_limit {
            let token_period = format_period(tl.unit, tl.number);
            let token_percentage = tl.percentage as f64;
            let token_reset_time = tl.next_reset_time
                .map(format_timestamp_js)
                .unwrap_or_else(|| "未知".to_string());
            (token_percentage, token_period, token_reset_time)
        } else {
            (0.0, "未知".to_string(), "未知".to_string())
        };

    // 处理周 token 额度
    let (weekly_token_percentage, weekly_token_period, weekly_token_reset_time) =
        if let Some(tl) = weekly_token_limit {
            let weekly_token_period = format_period(tl.unit, tl.number);
            let weekly_token_percentage = tl.percentage as f64;
            let weekly_token_reset_time = tl.next_reset_time
                .map(format_timestamp_js)
                .unwrap_or_else(|| "未知".to_string());
            (weekly_token_percentage, weekly_token_period, weekly_token_reset_time)
        } else {
            (0.0, "未知".to_string(), "未知".to_string())
        };

    let (mcp_total, mcp_used, mcp_remaining, mcp_percentage, mcp_reset_time, mcp_search, mcp_web, mcp_zread) =
        if let Some(tl) = time_limit {
            // usage 代表总配额，remaining 代表剩余配额
            let mcp_total = tl.usage.unwrap_or(0);
            let mcp_used = tl.usage.unwrap_or(0) - tl.remaining.unwrap_or(0);
            let mcp_remaining = tl.remaining.unwrap_or(0);
            let mcp_percentage = tl.percentage;
            let mcp_reset_time = tl.next_reset_time
                .map(format_timestamp_js)
                .unwrap_or_else(|| "未知".to_string());

            let mut mcp_search = 0;
            let mut mcp_web = 0;
            let mut mcp_zread = 0;

            if let Some(details) = &tl.usage_details {
                for detail in details {
                    match detail.model_code.as_str() {
                        "search-prime" => mcp_search = detail.usage,
                        "web-reader" => mcp_web = detail.usage,
                        "zread" => mcp_zread = detail.usage,
                        _ => {}
                    }
                }
            }

            (mcp_total, mcp_used, mcp_remaining, mcp_percentage, mcp_reset_time, mcp_search, mcp_web, mcp_zread)
        } else {
            (0, 0, 0, 0, "未知".to_string(), 0, 0, 0)
        };

    ZhipuUsageInfo {
        token_percentage,
        token_period,
        token_reset_time,
        weekly_token_percentage,
        weekly_token_period,
        weekly_token_reset_time,
        mcp_total,
        mcp_used,
        mcp_remaining,
        mcp_percentage,
        mcp_reset_time,
        mcp_search,
        mcp_web,
        mcp_zread,
    }
}

fn read_cookies(cookie_path: &Path) -> Result<Vec<serde_json::Value>> {
    if !cookie_path.exists() {
        return Err(AppError::Auth("未找到 cookies，请先登录".to_string()));
    }
    let cookie_content = std::fs::read_to_string(cookie_path)?;
    let cookies: Vec<serde_json::Value> = serde_json::from_str(&cookie_content)?;
    Ok(cookies)
}

fn find_cookie_value<'a>(cookies: &'a [serde_json::Value], name: &str) -> Option<&'a str> {
    cookies
        .iter()
        .find(|c| c["name"] == name)
        .and_then(|c| c["value"].as_str())
}
