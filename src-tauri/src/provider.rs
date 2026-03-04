use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use tauri::{menu::IsMenuItem, AppHandle, Wry};

use crate::error::Result;

pub fn get_xdg_data_dir() -> PathBuf {
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".local/share")
}

pub fn get_app_data_dir() -> PathBuf {
    get_xdg_data_dir().join("am-i-in-debt")
}

pub trait UsageInfo: Send + Sync + 'static {
    fn provider_id(&self) -> &'static str;
    fn is_token_exhausted(&self) -> bool;
    fn render_menu_items(
        &self,
        app: &AppHandle,
        is_selected: bool,
    ) -> Vec<Box<dyn IsMenuItem<Wry>>>;
    
    fn clone_boxed(&self) -> Box<dyn UsageInfo>;
}

pub trait Provider: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn login_script_arg(&self) -> &'static str;
    fn auth_token_name(&self) -> &'static str;
    
    fn data_dir(&self) -> PathBuf {
        get_app_data_dir().join(self.id())
    }
    
    fn cookie_path(&self) -> PathBuf {
        self.data_dir().join("cookies.json")
    }
    
    fn settings_path(&self) -> PathBuf {
        self.data_dir().join("settings.json")
    }
    
    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<Box<dyn UsageInfo>>> + Send + 'static>>;
}

pub struct ProviderRegistry(pub &'static dyn Provider);

inventory::collect!(ProviderRegistry);

pub fn format_progress_bar(percentage: f64) -> String {
    let pct = percentage.min(100.0);
    let filled = (pct / 10.0) as usize;
    let bar: String = (0..10).map(|i| if i < filled { '█' } else { '░' }).collect();
    format!("[{}] {:.0}%", bar, pct)
}

pub fn format_timestamp_js(ts_ms: i64) -> String {
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

pub fn format_iso_time(iso_time: &str) -> String {
    if iso_time.is_empty() {
        return "未知".to_string();
    }
    use chrono_tz::Asia::Shanghai;
    chrono::DateTime::parse_from_rfc3339(iso_time)
        .map(|dt| {
            let utc_dt: chrono::DateTime<chrono::Utc> = dt.with_timezone(&chrono::Utc);
            utc_dt.with_timezone(&Shanghai)
                .format("%y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|_| "未知".to_string())
}

pub fn format_last_update_time(timestamp_ms: i64) -> String {
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
