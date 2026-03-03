#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    Manager,
};

// 获取 XDG 数据目录
fn get_xdg_data_dir() -> PathBuf {
    // 优先使用 XDG_DATA_HOME 环境变量
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data);
    }

    // 如果没有设置，使用 XDG 规范的默认值 ~/.local/share
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".local/share")
}

// 获取应用数据目录（使用 XDG 规范）
fn get_app_data_dir() -> PathBuf {
    get_xdg_data_dir().join("coding-plan-usage")
}

// Coding Plan 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CodingPlan {
    Zhipu,
    Kimi,
}

impl CodingPlan {
    fn name(&self) -> &str {
        match self {
            CodingPlan::Zhipu => "智谱",
            CodingPlan::Kimi => "Kimi",
        }
    }

    fn id(&self) -> &str {
        match self {
            CodingPlan::Zhipu => "zhipu",
            CodingPlan::Kimi => "kimi",
        }
    }

    fn data_dir(&self) -> PathBuf {
        get_app_data_dir().join(format!("{}-coding-plan", self.id()))
    }
}

// ========== 智谱数据结构 ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ZhipuUsageInfo {
    // Token 额度（每 x 小时）
    token_total: i64,
    token_used: i64,
    token_remaining: i64,
    token_percentage: f64,
    token_hours: i64,  // 每 x 小时
    token_reset_time: String,  // 格式: YY-MM-DD HH:mm:ss

    // MCP 额度（每月）
    mcp_total: i64,
    mcp_used: i64,
    mcp_remaining: i64,
    mcp_percentage: i64,
    mcp_reset_time: String,  // 格式: YY-MM-DD HH:mm:ss

    // MCP 工具详情
    mcp_search: i64,    // 搜索 mcp
    mcp_web: i64,       // 网页读取 mcp
    mcp_zread: i64,     // zread
}

// 智谱 API 响应结构
#[derive(Debug, Serialize, Deserialize)]
struct ZhipuApiResponse {
    code: i32,
    data: Option<ZhipuQuotaData>,
    msg: Option<String>,
    success: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZhipuQuotaData {
    limits: Vec<ZhipuLimit>,
    level: String,
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
    next_reset_time: i64,
    #[serde(rename = "usageDetails")]
    usage_details: Option<Vec<ZhipuUsageDetail>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZhipuUsageDetail {
    #[serde(rename = "modelCode")]
    model_code: String,
    usage: i64,
}

// ========== Kimi 数据结构 ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KimiUsageInfo {
    // 5小时窗口数据
    hourly_total: i64,
    hourly_used: i64,
    hourly_remaining: i64,
    hourly_percentage: f64,
    hourly_reset_time: String,  // 格式: YY-MM-DD HH:mm:ss
    hourly_window: i64,  // 小时数（如 5）

    // 每周数据
    weekly_total: i64,
    weekly_used: i64,
    weekly_remaining: i64,
    weekly_percentage: f64,
    weekly_reset_time: String,  // 格式: YY-MM-DD HH:mm:ss
}

// Kimi API 响应结构
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

// ========== 应用状态 ==========

// 统一的使用信息枚举
#[derive(Clone)]
enum UsageInfo {
    Zhipu(ZhipuUsageInfo),
    Kimi(KimiUsageInfo),
}

impl UsageInfo {
    fn plan_id(&self) -> &str {
        match self {
            UsageInfo::Zhipu(_) => "zhipu",
            UsageInfo::Kimi(_) => "kimi",
        }
    }

    fn plan_name(&self) -> &str {
        match self {
            UsageInfo::Zhipu(_) => "智谱",
            UsageInfo::Kimi(_) => "Kimi",
        }
    }

    fn is_zhipu(&self) -> bool {
        matches!(self, UsageInfo::Zhipu(_))
    }

    fn is_kimi(&self) -> bool {
        matches!(self, UsageInfo::Kimi(_))
    }
}

// 应用状态
struct AppState {
    usage_info: Arc<Mutex<Vec<UsageInfo>>>,
    tray: Arc<Mutex<Option<TrayIcon>>>,
    last_update_time: Arc<Mutex<Option<i64>>>,
}

// ========== 工具函数 ==========

// 格式化数字
fn format_number(num: i64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

// 格式化进度条
fn format_progress_bar(percentage: f64) -> String {
    let pct = percentage.min(100.0);
    let filled = (pct / 10.0) as usize;
    let bar: String = (0..10).map(|i| if i < filled { '█' } else { '░' }).collect();
    format!("[{}] {:.0}%", bar, pct)
}

// 将 JavaScript 时间戳（毫秒）转换为格式化时间字符串（上海时区）
fn format_timestamp_js(ts_ms: i64) -> String {
    if ts_ms == 0 {
        return "未知".to_string();
    }
    let ts_sec = ts_ms / 1000;
    use chrono_tz::Asia::Shanghai;
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts_sec, 0)
        .unwrap_or_default()
        .with_timezone(&Shanghai)
        .format("%y-%m-%d %H:%M:%S")
        .to_string()
}

// 将 ISO 8601 时间字符串转换为格式化时间字符串（上海时区）
fn format_iso_time(iso_time: &str) -> String {
    if iso_time.is_empty() {
        return "未知".to_string();
    }
    use chrono_tz::Asia::Shanghai;
    chrono::DateTime::parse_from_rfc3339(iso_time)
        .map(|dt| {
            // 先转换为 UTC，再转换为上海时区
            let utc_dt: chrono::DateTime<chrono::Utc> = dt.with_timezone(&chrono::Utc);
            utc_dt.with_timezone(&Shanghai)
                .format("%y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|_| "未知".to_string())
}

// 格式化最近更新时间（显示 HH:MM:SS）
fn format_last_update_time(timestamp_ms: i64) -> String {
    if timestamp_ms == 0 {
        return "".to_string();
    }
    let ts_sec = timestamp_ms / 1000;
    use chrono_tz::Asia::Shanghai;
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts_sec, 0)
        .unwrap_or_default()
        .with_timezone(&Shanghai)
        .format("%H:%M:%S")
        .to_string()
}

// 更新最后更新时间
fn update_last_update_time(state: &tauri::State<AppState>) {
    use chrono::Utc;
    let now = Utc::now();
    let timestamp_ms = now.timestamp_millis();
    *state.last_update_time.lock().unwrap() = Some(timestamp_ms);
}

// ========== API 调用函数 ==========

// 获取智谱使用情况
async fn fetch_zhipu_usage(cookie_path: &std::path::Path) -> Result<ZhipuUsageInfo, String> {
    if !cookie_path.exists() {
        return Err("未找到 cookies，请先登录".to_string());
    }

    let cookie_content = fs::read_to_string(cookie_path)
        .map_err(|e| format!("读取 cookies 失败: {}", e))?;

    let cookies: Vec<serde_json::Value> = serde_json::from_str(&cookie_content)
        .map_err(|e| format!("解析 cookies 失败: {}", e))?;

    let token = cookies
        .iter()
        .find(|c| c["name"] == "bigmodel_token_production")
        .and_then(|c| c["value"].as_str())
        .ok_or("未找到认证 token")?;

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get("https://bigmodel.cn/api/monitor/usage/quota/limit")
        .header("authorization", token)
        .header("referer", "https://bigmodel.cn/usercenter/glm-coding/usage")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let api_response: ZhipuApiResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析响应失败: {} - {}", e, response_text))?;

    if api_response.code != 200 {
        return Err(api_response.msg.unwrap_or("获取数据失败".to_string()));
    }

    let data = api_response.data.ok_or("响应数据为空")?;

    // 查找两种类型的配额
    let time_limit = data.limits.iter().find(|l| l.limit_type == "TIME_LIMIT");
    let token_limit = data.limits.iter().find(|l| l.limit_type == "TOKENS_LIMIT");

    let token_limit = token_limit.ok_or("未找到 token 配额信息")?;

    // ========== Token 额度（每 x 小时）==========
    // token_limit.unit = 3 (小时单位), token_limit.number = 5
    // 所以是每 (3 * 5) / 3 = 5 小时
    let token_hours = (token_limit.unit * token_limit.number) / 3;
    let token_total = token_limit.unit * token_limit.number * 1_000_000;
    let token_used = (token_total as f64 * token_limit.percentage as f64 / 100.0) as i64;
    let token_remaining = token_total - token_used;
    let token_percentage = token_limit.percentage as f64;
    let token_reset_time = format_timestamp_js(token_limit.next_reset_time);

    // ========== MCP 额度（每月）==========
    let (mcp_total, mcp_used, mcp_remaining, mcp_percentage, mcp_reset_time, mcp_search, mcp_web, mcp_zread) =
        if let Some(tl) = time_limit {
            let mcp_total = tl.usage.unwrap_or(0) + tl.remaining.unwrap_or(0);
            let mcp_used = tl.usage.unwrap_or(0);
            let mcp_remaining = tl.remaining.unwrap_or(0);
            let mcp_percentage = tl.percentage;
            let mcp_reset_time = format_timestamp_js(tl.next_reset_time);

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

    Ok(ZhipuUsageInfo {
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
    })
}

// 获取 Kimi 使用情况
async fn fetch_kimi_usage(cookie_path: &std::path::Path) -> Result<KimiUsageInfo, String> {
    if !cookie_path.exists() {
        return Err("未找到 cookies，请先登录".to_string());
    }

    let cookie_content = fs::read_to_string(cookie_path)
        .map_err(|e| format!("读取 cookies 失败: {}", e))?;

    let cookies: Vec<serde_json::Value> = serde_json::from_str(&cookie_content)
        .map_err(|e| format!("解析 cookies 失败: {}", e))?;

    // 查找 kimi-auth cookie
    let auth_token = cookies
        .iter()
        .find(|c| c["name"] == "kimi-auth")
        .and_then(|c| c["value"].as_str())
        .ok_or("未找到 kimi-auth cookie，请重新登录")?
        .to_string();

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {}", e))?;

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
        .map_err(|e| format!("请求失败: {}", e))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    // 先检查是否返回 unauthenticated 错误
    if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&response_text) {
        if error_response.get("code").and_then(|c| c.as_str()) == Some("unauthenticated") {
            // Token 无效，删除 cookies 文件并返回错误
            let _ = fs::remove_file(cookie_path);
            return Err("Token 已失效，请重新登录".to_string());
        }
    }

    let api_response: KimiApiResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析响应失败: {} - {}", e, response_text))?;

    // 查找 FEATURE_CODING 类型的使用情况
    let usage = api_response.usages
        .iter()
        .find(|u| u.scope == "FEATURE_CODING")
        .ok_or("未找到 CODING 使用情况")?;

    // ========== 5小时窗口数据 ==========
    let limit_detail = usage.limits.first()
        .ok_or("未找到 limits 数据")?
        .detail.clone();
    let window = &usage.limits.first().unwrap().window;

    let hourly_total = limit_detail.limit_value.parse::<i64>()
        .map_err(|e| format!("解析 limit 失败: {}", e))?;
    let hourly_remaining = limit_detail.remaining.parse::<i64>()
        .map_err(|e| format!("解析 remaining 失败: {}", e))?;
    let hourly_used = hourly_total - hourly_remaining;
    let hourly_percentage = if hourly_total > 0 { (hourly_used as f64 / hourly_total as f64) * 100.0 } else { 0.0 };
    let hourly_reset_time = format_iso_time(&limit_detail.reset_time);
    let hourly_window = window.duration / 60; // 转换为小时

    // ========== 每周数据 ==========
    let weekly_detail = usage.detail.clone();
    let weekly_total = weekly_detail.limit_value.parse::<i64>()
        .map_err(|e| format!("解析 weekly limit 失败: {}", e))?;
    let weekly_used = weekly_detail.used.parse::<i64>()
        .map_err(|e| format!("解析 weekly used 失败: {}", e))?;
    let weekly_remaining = weekly_detail.remaining.parse::<i64>()
        .map_err(|e| format!("解析 weekly remaining 失败: {}", e))?;
    let weekly_percentage = if weekly_total > 0 { (weekly_used as f64 / weekly_total as f64) * 100.0 } else { 0.0 };
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

// 获取指定 plan 的使用情况
async fn fetch_usage_info(plan: CodingPlan) -> Result<UsageInfo, String> {
    let cookie_path = plan.data_dir().join("cookies.json");

    match plan {
        CodingPlan::Zhipu => fetch_zhipu_usage(&cookie_path).await.map(UsageInfo::Zhipu),
        CodingPlan::Kimi => fetch_kimi_usage(&cookie_path).await.map(UsageInfo::Kimi),
    }
}

// 执行登录脚本
fn run_login_script(plan: CodingPlan) -> Result<(), String> {
    let output_dir = plan.data_dir();

    // 创建目录（如果不存在）
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    // 获取 sidecar 可执行文件路径
    let sidecar_name = match plan {
        CodingPlan::Zhipu => "get-zhipu-cookies",
        CodingPlan::Kimi => "get-kimi-cookies",
    };

    let sidecar_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bin")
        .join(sidecar_name);

    if !sidecar_path.exists() {
        return Err(format!("Sidecar 文件不存在: {:?}", sidecar_path));
    }

    // 执行 sidecar
    let output = std::process::Command::new(&sidecar_path)
        .env("COOKIES_OUTPUT_PATH", output_dir.to_string_lossy().to_string())
        .output()
        .map_err(|e| format!("执行 sidecar 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("登录失败: {}\n{}", stdout, stderr));
    }

    // 打印输出
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

// 更新菜单
fn update_menu(app: &tauri::AppHandle, usage_list: &[UsageInfo]) {
    let state: tauri::State<AppState> = app.state();
    let mut info = state.usage_info.lock().unwrap();
    *info = usage_list.to_vec();

    // 更新最后更新时间
    update_last_update_time(&state);

    // 获取最后更新时间
    let last_update = state.last_update_time.lock().unwrap();
    let update_time_suffix = if let Some(ts) = *last_update {
        format!(" ({})", format_last_update_time(ts))
    } else {
        "".to_string()
    };
    drop(last_update);

    // 检查哪些 plan 已登录
    let zhipu_logged_in = usage_list.iter().any(|u| u.is_zhipu());
    let kimi_logged_in = usage_list.iter().any(|u| u.is_kimi());

    // 根据登录状态构建不同的菜单
    let menu = if usage_list.is_empty() {
        // 没有登录任何 plan
        let header = MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
        let sep1 = PredefinedMenuItem::separator(app).unwrap();
        let login_zhipu = MenuItem::with_id(app, "login-zhipu", "登录智谱 Coding Plan", true, None::<&str>).unwrap();
        let login_kimi = MenuItem::with_id(app, "login-kimi", "登录 Kimi Coding Plan", true, None::<&str>).unwrap();
        let sep2 = PredefinedMenuItem::separator(app).unwrap();
        let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();

        Menu::with_items(
            app,
            &[&header as &dyn tauri::menu::IsMenuItem<_>, &sep1, &login_zhipu, &login_kimi, &sep2, &quit],
        )
    } else {
        // 先创建所有菜单项
        let header = MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
        let sep1 = PredefinedMenuItem::separator(app).unwrap();

        let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<_> + '_>> = vec![
            Box::new(header),
            Box::new(sep1),
        ];

        // 为每个 plan 添加使用情况或登录选项
        for plan in [CodingPlan::Zhipu, CodingPlan::Kimi] {
            if let Some(usage) = usage_list.iter().find(|u| u.plan_id() == plan.id()) {
                match usage {
                    UsageInfo::Zhipu(zhipu_info) => {
                        // 智谱 Coding Plan
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-header", plan.id()),
                            format!("{} Coding Plan", plan.name()),
                            true,
                            None::<&str>
                        ).unwrap()));

                        // Token 额度（每 x 小时）
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-token-title", plan.id()),
                            format!("Token 额度（每 {} 小时）", zhipu_info.token_hours),
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-token-bar", plan.id()),
                            format_progress_bar(zhipu_info.token_percentage),
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-token-reset", plan.id()),
                            format!("重置: {}", zhipu_info.token_reset_time),
                            false,
                            None::<&str>
                        ).unwrap()));

                        // 分隔线
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-sep", plan.id()),
                            "-".repeat(25),
                            false,
                            None::<&str>
                        ).unwrap()));

                        // MCP 额度（每月）
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-mcp-title", plan.id()),
                            "MCP 额度（每月）",
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-mcp-bar", plan.id()),
                            format_progress_bar(zhipu_info.mcp_percentage as f64),
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-mcp-detail", plan.id()),
                            format!("搜索: {} | 网页: {} | 阅读: {}",
                                zhipu_info.mcp_search, zhipu_info.mcp_web, zhipu_info.mcp_zread),
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-mcp-reset", plan.id()),
                            format!("重置: {}", zhipu_info.mcp_reset_time),
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
                    }
                    UsageInfo::Kimi(kimi_info) => {
                        // Kimi Coding Plan
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-header", plan.id()),
                            format!("{} Coding Plan", plan.name()),
                            true,
                            None::<&str>
                        ).unwrap()));

                        // Token 额度（每 x 小时）
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-hourly-title", plan.id()),
                            format!("Token 额度（每 {} 小时）", kimi_info.hourly_window),
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-hourly-bar", plan.id()),
                            format_progress_bar(kimi_info.hourly_percentage),
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-hourly-reset", plan.id()),
                            format!("重置: {}", kimi_info.hourly_reset_time),
                            false,
                            None::<&str>
                        ).unwrap()));

                        // 分隔线
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-sep", plan.id()),
                            "-".repeat(25),
                            false,
                            None::<&str>
                        ).unwrap()));

                        // Token 额度（每周）
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-weekly-title", plan.id()),
                            "Token 额度（每周）",
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-weekly-bar", plan.id()),
                            format_progress_bar(kimi_info.weekly_percentage),
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(MenuItem::with_id(
                            app,
                            format!("{}-weekly-reset", plan.id()),
                            format!("重置: {}", kimi_info.weekly_reset_time),
                            false,
                            None::<&str>
                        ).unwrap()));
                        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
                    }
                }
            } else {
                items.push(Box::new(MenuItem::with_id(
                    app,
                    format!("login-{}", plan.id()),
                    format!("登录{} Coding Plan", plan.name()),
                    true,
                    None::<&str>
                ).unwrap()));
            }
        }

        // 添加通用菜单项
        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
        items.push(Box::new(MenuItem::with_id(app, "refresh", format!("刷新{}", update_time_suffix), true, None::<&str>).unwrap()));

        if zhipu_logged_in {
            items.push(Box::new(MenuItem::with_id(app, "relogin-zhipu", "重新登录智谱", true, None::<&str>).unwrap()));
        }
        if kimi_logged_in {
            items.push(Box::new(MenuItem::with_id(app, "relogin-kimi", "重新登录 Kimi", true, None::<&str>).unwrap()));
        }

        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
        items.push(Box::new(MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap()));

        // 提取引用
        let items_refs: Vec<&dyn tauri::menu::IsMenuItem<_>> = items.iter().map(|item| item.as_ref()).collect();
        Menu::with_items(app, &items_refs)
    };

    if let Ok(menu) = menu {
        let tray = state.tray.lock().unwrap();
        if let Some(tray) = tray.as_ref() {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn main() {
    // 需要在 setup 外部创建共享状态
    let tray_state = Arc::new(Mutex::new(None::<TrayIcon>));

    tauri::Builder::default()
        .manage(AppState {
            usage_info: Arc::new(Mutex::new(Vec::new())),
            tray: tray_state.clone(),
            last_update_time: Arc::new(Mutex::new(None)),
        })
        .setup(|app| {
            // 加载托盘图标
            let tray_icon = include_bytes!("../icons/icon.png");
            let icon = tauri::image::Image::from_bytes(&tray_icon.to_vec())
                .expect("Failed to load tray icon");

            // 初始菜单
            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap(),
                    &PredefinedMenuItem::separator(app).unwrap(),
                    &MenuItem::with_id(app, "status", "加载中...", false, None::<&str>).unwrap(),
                    &PredefinedMenuItem::separator(app).unwrap(),
                    &MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap(),
                ],
            )?;

            // 创建托盘图标
            let tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app: &tauri::AppHandle, event| {
                    let event_id = event.id.as_ref();

                    // 处理登录相关事件
                    if event_id == "login-zhipu" || event_id == "relogin-zhipu" {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = run_login_script(CodingPlan::Zhipu) {
                                eprintln!("登录智谱失败: {}", e);
                            } else {
                                let cookie_path = CodingPlan::Zhipu.data_dir().join("cookies.json");
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                match fetch_zhipu_usage(&cookie_path).await {
                                    Ok(info) => {
                                        // 更新菜单，添加新的使用情况
                                        let state: tauri::State<AppState> = app_handle.state();
                                        let mut usage_list = state.usage_info.lock().unwrap();
                                        usage_list.retain(|u| !u.is_zhipu());
                                        usage_list.push(UsageInfo::Zhipu(info));
                                        let usage_vec = usage_list.clone();
                                        drop(usage_list);
                                        update_menu(&app_handle, &usage_vec);
                                    }
                                    Err(e) => {
                                        eprintln!("获取智谱使用情况失败: {}", e);
                                    }
                                }
                            }
                        });
                    } else if event_id == "login-kimi" || event_id == "relogin-kimi" {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = run_login_script(CodingPlan::Kimi) {
                                eprintln!("登录 Kimi 失败: {}", e);
                            } else {
                                let cookie_path = CodingPlan::Kimi.data_dir().join("cookies.json");
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                match fetch_kimi_usage(&cookie_path).await {
                                    Ok(info) => {
                                        // 更新菜单，添加新的使用情况
                                        let state: tauri::State<AppState> = app_handle.state();
                                        let mut usage_list = state.usage_info.lock().unwrap();
                                        usage_list.retain(|u| !u.is_kimi());
                                        usage_list.push(UsageInfo::Kimi(info));
                                        let usage_vec = usage_list.clone();
                                        drop(usage_list);
                                        update_menu(&app_handle, &usage_vec);
                                    }
                                    Err(e) => {
                                        eprintln!("获取 Kimi 使用情况失败: {}", e);
                                    }
                                }
                            }
                        });
                    } else if event_id == "refresh" {
                        // 刷新按钮
                    } else if event_id == "quit" {
                        std::process::exit(0);
                    }
                })
                .build(app)?;

            // 保存 tray 到状态
            let state: tauri::State<AppState> = app.state();
            *state.tray.lock().unwrap() = Some(tray);

            // 启动时自动获取所有 plan 的使用情况
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut usage_list = Vec::new();

                // 尝试获取智谱使用情况
                let zhipu_path = CodingPlan::Zhipu.data_dir().join("cookies.json");
                if let Ok(info) = fetch_zhipu_usage(&zhipu_path).await {
                    usage_list.push(UsageInfo::Zhipu(info));
                }

                // 尝试获取 Kimi 使用情况
                let kimi_path = CodingPlan::Kimi.data_dir().join("cookies.json");
                if let Ok(info) = fetch_kimi_usage(&kimi_path).await {
                    usage_list.push(UsageInfo::Kimi(info));
                }

                update_menu(&app_handle, &usage_list);
            });

            // 每 30 秒自动刷新
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                    let mut usage_list = Vec::new();

                    // 尝试获取智谱使用情况
                    let zhipu_path = CodingPlan::Zhipu.data_dir().join("cookies.json");
                    if let Ok(info) = fetch_zhipu_usage(&zhipu_path).await {
                        usage_list.push(UsageInfo::Zhipu(info));
                    }

                    // 尝试获取 Kimi 使用情况
                    let kimi_path = CodingPlan::Kimi.data_dir().join("cookies.json");
                    if let Ok(info) = fetch_kimi_usage(&kimi_path).await {
                        usage_list.push(UsageInfo::Kimi(info));
                    }

                    update_menu(&app_handle, &usage_list);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
