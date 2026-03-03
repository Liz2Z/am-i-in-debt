use crate::error::{AppError, Result};
use crate::models::{
    format_iso_time, UsageInfo, KimiApiResponse, KimiUsageInfo,
};
use std::fs;
use std::path::Path;

pub async fn fetch_kimi_usage(cookie_path: &Path) -> Result<UsageInfo> {
    let cookies = super::read_cookies(cookie_path)?;

    let auth_token = super::find_cookie_value(&cookies, "kimi-auth")
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

    let info = build_kimi_usage_info(usage)?;
    Ok(UsageInfo::Kimi(info))
}

fn build_kimi_usage_info(usage: &crate::models::KimiUsage) -> Result<KimiUsageInfo> {
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
