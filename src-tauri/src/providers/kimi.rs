use std::fs;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::future::Future;

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, IsMenuItem, MenuItem, PredefinedMenuItem},
    AppHandle, Wry,
};

use crate::error::{AppError, Result};
use super::{format_progress_bar, format_iso_time, Provider, UsageInfo};

pub const KIMI_ID: &str = "kimi-coding-plan";
pub const KIMI_DISPLAY_NAME: &str = "Kimi";
pub const KIMI_LOGIN_ARG: &str = "kimi";
pub const KIMI_AUTH_TOKEN: &str = "kimi-auth";

pub struct KimiProvider;

impl Provider for KimiProvider {
    fn id(&self) -> &'static str { KIMI_ID }
    fn display_name(&self) -> &'static str { KIMI_DISPLAY_NAME }
    fn login_script_arg(&self) -> &'static str { KIMI_LOGIN_ARG }
    fn auth_token_name(&self) -> &'static str { KIMI_AUTH_TOKEN }
    
    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<UsageInfo>> + Send + 'static>> {
        Box::pin(async move {
            let cookies = read_cookies(&cookie_path)?;
            let auth_token = find_cookie_value(&cookies, KIMI_AUTH_TOKEN)
                .ok_or_else(|| AppError::Auth("未找到 kimi-auth cookie，请重新登录".to_string()))?
                .to_string();
            
            let client = reqwest::Client::builder()
                .build()
                .map_err(|e| AppError::Http(e.to_string()))?;
            
            let response = client
                .post("https://www.kimi.com/apiv2/kimi.gateway.billing.v1.BillingService/GetUsages")
                .header("accept", "*/*")
                .header("authorization", format!("Bearer {}", auth_token))
                .header("content-type", "application/json")
                .header("connect-protocol-version", "1")
                .header("origin", "https://www.kimi.com")
                .header("referer", "https://www.kimi.com/code/console")
                .header("x-language", "zh-CN")
                .header("x-msh-platform", "web")
                .header("x-msh-version", "1.0.0")
                .header("r-timezone", "Asia/Shanghai")
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                .json(&serde_json::json!({"scope": ["FEATURE_CODING"]}))
                .send()
                .await
                .map_err(|e| AppError::Http(e.to_string()))?;
            
            let response_text = response
                .text()
                .await
                .map_err(|e| AppError::Http(e.to_string()))?;
            
            if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if error_response.get("code").and_then(|c| c.as_str()) == Some("unauthenticated") {
                    let _ = fs::remove_file(cookie_path);
                    return Err(AppError::Auth("Token 已失效，请重新登录".to_string()));
                }
            }
            
            let api_response: KimiApiResponse = serde_json::from_str(&response_text)
                .map_err(|e| AppError::Parse(format!("{} - {}", e, response_text)))?;
            
            let usage = api_response
                .usages
                .iter()
                .find(|u| u.scope == "FEATURE_CODING")
                .ok_or(AppError::Parse("未找到 CODING 使用情况".to_string()))?;
            
            let info = build_usage_info(usage)?;
            Ok(UsageInfo::Kimi(info))
        })
    }
    
    fn render_menu_items(
        &self,
        app: &AppHandle,
        usage: &UsageInfo,
        is_selected: bool,
    ) -> Vec<Box<dyn IsMenuItem<Wry>>> {
        let UsageInfo::Kimi(info) = usage else { return vec![] };
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
            format!("{}-hourly-title", self.id()),
            format!("Token 额度（每 {} 小时）", info.hourly_window),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-hourly-bar", self.id()),
            format_progress_bar(info.hourly_percentage),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-hourly-reset", self.id()),
            format!("重置: {}", info.hourly_reset_time),
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
            format!("{}-weekly-title", self.id()),
            "Token 额度（每周）",
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-weekly-bar", self.id()),
            format_progress_bar(info.weekly_percentage),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-weekly-reset", self.id()),
            format!("重置: {}", info.weekly_reset_time),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
        
        items
    }
}

pub static KIMI: KimiProvider = KimiProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
struct KimiApiResponse {
    usages: Vec<KimiUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KimiUsage {
    scope: String,
    detail: KimiUsageDetail,
    limits: Vec<KimiUsageLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KimiUsageDetail {
    #[serde(rename = "limit")]
    limit_value: String,
    used: String,
    remaining: String,
    #[serde(rename = "resetTime")]
    reset_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct KimiUsageLimit {
    window: KimiWindow,
    detail: KimiUsageLimitDetail,
}

#[derive(Debug, Serialize, Deserialize)]
struct KimiWindow {
    duration: i64,
    #[serde(rename = "timeUnit")]
    time_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KimiUsageLimitDetail {
    #[serde(rename = "limit")]
    limit_value: String,
    remaining: String,
    #[serde(rename = "resetTime")]
    reset_time: String,
}

fn build_usage_info(usage: &KimiUsage) -> Result<KimiUsageInfo> {
    let limit_detail = usage
        .limits
        .first()
        .ok_or(AppError::Parse("未找到 limits 数据".to_string()))?
        .detail
        .clone();
    let window = &usage.limits.first().unwrap().window;
    
    let hourly_total = limit_detail
        .limit_value
        .parse::<i64>()
        .map_err(|e| AppError::Parse(format!("解析 limit 失败: {}", e)))?;
    let hourly_remaining = limit_detail
        .remaining
        .parse::<i64>()
        .map_err(|e| AppError::Parse(format!("解析 remaining 失败: {}", e)))?;
    let hourly_used = hourly_total - hourly_remaining;
    let hourly_percentage = if hourly_total > 0 {
        (hourly_used as f64 / hourly_total as f64) * 100.0
    } else {
        0.0
    };
    let hourly_reset_time = format_iso_time(&limit_detail.reset_time);
    let hourly_window = window.duration / 60;
    
    let weekly_detail = usage.detail.clone();
    let weekly_total = weekly_detail
        .limit_value
        .parse::<i64>()
        .map_err(|e| AppError::Parse(format!("解析 weekly limit 失败: {}", e)))?;
    let weekly_used = weekly_detail
        .used
        .parse::<i64>()
        .map_err(|e| AppError::Parse(format!("解析 weekly used 失败: {}", e)))?;
    let weekly_remaining = weekly_detail
        .remaining
        .parse::<i64>()
        .map_err(|e| AppError::Parse(format!("解析 weekly remaining 失败: {}", e)))?;
    let weekly_percentage = if weekly_total > 0 {
        (weekly_used as f64 / weekly_total as f64) * 100.0
    } else {
        0.0
    };
    let weekly_reset_time = format_iso_time(&weekly_detail.reset_time);
    
    Ok(KimiUsageInfo {
        hourly_total,
        hourly_used,
        hourly_remaining,
        hourly_percentage,
        hourly_reset_time,
        hourly_window,
        weekly_total,
        weekly_used,
        weekly_remaining,
        weekly_percentage,
        weekly_reset_time,
    })
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
