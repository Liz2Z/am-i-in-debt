mod zhipu;
mod kimi;

pub use zhipu::*;
pub use kimi::*;

use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::LazyLock;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Provider {
    pub id: &'static str,
    pub display_name: &'static str,
    pub login_script_arg: &'static str,
}

impl Provider {
    pub const ZHIPU: Provider = Provider {
        id: "zhipu-coding-plan",
        display_name: "智谱",
        login_script_arg: "zhipu",
    };

    pub const KIMI: Provider = Provider {
        id: "kimi-coding-plan",
        display_name: "Kimi",
        login_script_arg: "kimi",
    };

    pub fn data_dir(&self) -> PathBuf {
        get_app_data_dir().join(self.id)
    }

    pub fn cookie_path(&self) -> PathBuf {
        self.data_dir().join("cookies.json")
    }
}

pub static PROVIDER_REGISTRY: LazyLock<HashMap<&'static str, Provider>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(Provider::ZHIPU.id, Provider::ZHIPU);
    map.insert(Provider::KIMI.id, Provider::KIMI);
    map
});

pub static ALL_PROVIDERS: LazyLock<Vec<Provider>> = LazyLock::new(|| {
    vec![Provider::ZHIPU, Provider::KIMI]
});

pub fn get_provider_by_id(id: &str) -> Option<Provider> {
    PROVIDER_REGISTRY.get(id).cloned()
}

pub type CodingPlan = Provider;

pub trait MenuRenderable {
    fn provider(&self) -> Provider;
    fn render_menu_items(
        &self,
        app: &tauri::AppHandle,
        provider: Provider,
        is_selected: bool,
    ) -> Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>>;
}

#[derive(Debug, Clone)]
pub enum UsageInfo {
    Zhipu(ZhipuUsageInfo),
    Kimi(KimiUsageInfo),
}

impl UsageInfo {
    pub fn provider(&self) -> Provider {
        match self {
            UsageInfo::Zhipu(info) => info.provider(),
            UsageInfo::Kimi(info) => info.provider(),
        }
    }

    pub fn provider_id(&self) -> &'static str {
        self.provider().id
    }

    pub fn render_menu_items(
        &self,
        app: &tauri::AppHandle,
        is_selected: bool,
    ) -> Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> {
        match self {
            UsageInfo::Zhipu(info) => info.render_menu_items(app, info.provider(), is_selected),
            UsageInfo::Kimi(info) => info.render_menu_items(app, info.provider(), is_selected),
        }
    }
}

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
