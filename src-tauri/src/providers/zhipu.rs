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

pub struct ZhipuProvider;

impl Provider for ZhipuProvider {
    fn id(&self) -> &'static str { ZHIPU_ID }
    fn display_name(&self) -> &'static str { ZHIPU_DISPLAY_NAME }
    fn login_script_arg(&self) -> &'static str { ZHIPU_LOGIN_ARG }
    fn auth_token_name(&self) -> &'static str { ZHIPU_AUTH_TOKEN }
    
    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<UsageInfo>> + Send + 'static>> {
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
            let token_limit = data
                .limits
                .iter()
                .find(|l| l.limit_type == "TOKENS_LIMIT")
                .ok_or(AppError::Parse("未找到 token 配额信息".to_string()))?;
            
            let info = build_usage_info(token_limit, time_limit);
            Ok(UsageInfo::Zhipu(info))
        })
    }
    
    fn render_menu_items(
        &self,
        app: &AppHandle,
        usage: &UsageInfo,
        is_selected: bool,
    ) -> Vec<Box<dyn IsMenuItem<Wry>>> {
        let UsageInfo::Zhipu(info) = usage else { return vec![] };
        let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
        
        items.push(Box::new(CheckMenuItem::with_id(
            app,
            format!("select-{}", self.id()),
            format!("{} Coding Plan", self.display_name()),
            true,
            is_selected,
            None::<&str>,
        ).unwrap()));
        
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-token-title", self.id()),
            format!("Token 额度（每 {} 小时）", info.token_hours),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-token-bar", self.id()),
            format_progress_bar(info.token_percentage),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-token-reset", self.id()),
            format!("重置: {}", info.token_reset_time),
            false,
            None::<&str>,
        ).unwrap()));
        
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-sep", self.id()),
            "-".repeat(25),
            false,
            None::<&str>,
        ).unwrap()));
        
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-mcp-title", self.id()),
            "MCP 额度（每月）",
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-mcp-bar", self.id()),
            format_progress_bar(info.mcp_percentage as f64),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-mcp-detail", self.id()),
            format!("搜索: {} | 网页: {} | 阅读: {}", info.mcp_search, info.mcp_web, info.mcp_zread),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-mcp-reset", self.id()),
            format!("重置: {}", info.mcp_reset_time),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
        
        items
    }
}

pub static ZHIPU: ZhipuProvider = ZhipuProvider;

inventory::submit!(ProviderRegistry(&ZHIPU));

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn build_usage_info(token_limit: &ZhipuLimit, time_limit: Option<&ZhipuLimit>) -> ZhipuUsageInfo {
    let token_hours = (token_limit.unit * token_limit.number) / 3;
    let token_total = token_limit.unit * token_limit.number * 1_000_000;
    let token_used = (token_total as f64 * token_limit.percentage as f64 / 100.0) as i64;
    let token_remaining = token_total - token_used;
    let token_percentage = token_limit.percentage as f64;
    let token_reset_time = token_limit.next_reset_time
        .map(format_timestamp_js)
        .unwrap_or_else(|| "未知".to_string());
    
    let (mcp_total, mcp_used, mcp_remaining, mcp_percentage, mcp_reset_time, mcp_search, mcp_web, mcp_zread) =
        if let Some(tl) = time_limit {
            let mcp_total = tl.usage.unwrap_or(0) + tl.remaining.unwrap_or(0);
            let mcp_used = tl.usage.unwrap_or(0);
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
        token_total,
        token_used,
        token_remaining,
        token_percentage,
        token_hours,
        token_reset_time,
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
