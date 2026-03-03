use crate::error::{AppError, Result};
use crate::models::{
    format_timestamp_js, UsageInfo, ZhipuApiResponse, ZhipuLimit, ZhipuUsageInfo,
};
use log::{error, info};
use std::path::Path;

pub async fn fetch_zhipu_usage(cookie_path: &Path) -> Result<UsageInfo> {
    info!("开始获取智谱使用情况，cookie 路径: {:?}", cookie_path);
    
    let cookies = super::read_cookies(cookie_path)?;
    info!("读取到 {} 个 cookies", cookies.len());

    let token = super::find_cookie_value(&cookies, "bigmodel_token_production")
        .ok_or_else(|| {
            error!("未找到 bigmodel_token_production cookie");
            AppError::Auth("未找到认证 token".to_string())
        })?;
    
    info!("找到 token，长度: {}", token.len());

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| {
            error!("创建 HTTP 客户端失败: {}", e);
            AppError::Http(e.to_string())
        })?;

    info!("发送请求到 https://bigmodel.cn/api/monitor/usage/quota/limit");
    
    let response = client
        .get("https://bigmodel.cn/api/monitor/usage/quota/limit")
        .header("authorization", token)
        .header("referer", "https://bigmodel.cn/usercenter/glm-coding/usage")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| {
            error!("HTTP 请求失败: {}", e);
            AppError::Http(e.to_string())
        })?;

    let status = response.status();
    info!("响应状态码: {}", status);

    let response_text = response
        .text()
        .await
        .map_err(|e| {
            error!("读取响应文本失败: {}", e);
            AppError::Http(e.to_string())
        })?;

    info!("响应内容: {}", response_text);

    let api_response: ZhipuApiResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            error!("解析 JSON 失败: {} - 响应内容: {}", e, response_text);
            AppError::Parse(format!("{} - {}", e, response_text))
        })?;

    if api_response.code != 200 {
        return Err(AppError::Http(
            api_response.msg.unwrap_or("获取数据失败".to_string()),
        ));
    }

    let data = api_response.data.ok_or(AppError::Parse("响应数据为空".to_string()))?;

    let time_limit = data.limits.iter().find(|l| l.limit_type == "TIME_LIMIT");
    let token_limit = data
        .limits
        .iter()
        .find(|l| l.limit_type == "TOKENS_LIMIT")
        .ok_or(AppError::Parse("未找到 token 配额信息".to_string()))?;

    let info = build_zhipu_usage_info(token_limit, time_limit);
    Ok(UsageInfo::Zhipu(info))
}

fn build_zhipu_usage_info(
    token_limit: &ZhipuLimit,
    time_limit: Option<&ZhipuLimit>,
) -> ZhipuUsageInfo {
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
