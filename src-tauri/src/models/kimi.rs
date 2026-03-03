use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, MenuItem, PredefinedMenuItem},
    Wry,
};

use super::{format_progress_bar, MenuRenderable, Provider};

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

impl MenuRenderable for KimiUsageInfo {
    fn provider(&self) -> Provider {
        Provider::KIMI
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
            format!("{}-hourly-title", provider.id),
            format!("Token 额度（每 {} 小时）", self.hourly_window),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-hourly-bar", provider.id),
            format_progress_bar(self.hourly_percentage),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-hourly-reset", provider.id),
            format!("重置: {}", self.hourly_reset_time),
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
            format!("{}-weekly-title", provider.id),
            "Token 额度（每周）",
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-weekly-bar", provider.id),
            format_progress_bar(self.weekly_percentage),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("{}-weekly-reset", provider.id),
            format!("重置: {}", self.weekly_reset_time),
            false,
            None::<&str>,
        ).unwrap()));
        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));

        items
    }
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
