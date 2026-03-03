use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, MenuItem, PredefinedMenuItem},
    Wry,
};

use super::{format_progress_bar, MenuRenderable, Provider};

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

impl MenuRenderable for ZhipuUsageInfo {
    fn provider(&self) -> Provider {
        Provider::ZHIPU
    }

    fn render_menu_items(
        &self,
        app: &tauri::AppHandle,
        provider: Provider,
        is_selected: bool,
    ) -> Vec<Box<dyn tauri::menu::IsMenuItem<Wry>>> {
        let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry>>> = Vec::new();

        items.push(Box::new(CheckMenuItem::with_id(
            app,
            format!("select-{}", provider.id),
            format!("{} Coding Plan", provider.display_name),
            true,
            is_selected,
            None::<&str>,
        ).unwrap()));

        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-token-title", provider.id),
            format!("Token 额度（每 {} 小时）", self.token_hours),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-token-bar", provider.id),
            format_progress_bar(self.token_percentage),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-token-reset", provider.id),
            format!("重置: {}", self.token_reset_time),
            false,
            None::<&str>,
        ).unwrap()));

        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-sep", provider.id),
            "-".repeat(25),
            false,
            None::<&str>,
        ).unwrap()));

        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-mcp-title", provider.id),
            "MCP 额度（每月）",
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-mcp-bar", provider.id),
            format_progress_bar(self.mcp_percentage as f64),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-mcp-detail", provider.id),
            format!("搜索: {} | 网页: {} | 阅读: {}", self.mcp_search, self.mcp_web, self.mcp_zread),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-mcp-reset", provider.id),
            format!("重置: {}", self.mcp_reset_time),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));

        items
    }
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
    pub next_reset_time: Option<i64>,
    #[serde(rename = "usageDetails")]
    pub usage_details: Option<Vec<ZhipuUsageDetail>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZhipuUsageDetail {
    #[serde(rename = "modelCode")]
    pub model_code: String,
    pub usage: i64,
}
