mod zhipu;
mod kimi;

pub use zhipu::*;
pub use kimi::*;

use std::path::PathBuf;

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

pub const PROVIDER_ZHIPU: &str = "zhipu-coding-plan";
pub const PROVIDER_KIMI: &str = "kimi-coding-plan";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    Zhipu,
    Kimi,
}

impl Provider {
    pub const ALL: [Provider; 2] = [Provider::Zhipu, Provider::Kimi];
    
    pub fn provider_id(&self) -> &'static str {
        match self {
            Provider::Zhipu => PROVIDER_ZHIPU,
            Provider::Kimi => PROVIDER_KIMI,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Provider::Zhipu => "智谱",
            Provider::Kimi => "Kimi",
        }
    }

    pub fn login_script_arg(&self) -> &'static str {
        match self {
            Provider::Zhipu => "zhipu",
            Provider::Kimi => "kimi",
        }
    }

    pub fn data_dir(&self) -> PathBuf {
        get_app_data_dir().join(self.provider_id())
    }

    pub fn cookie_path(&self) -> PathBuf {
        self.data_dir().join("cookies.json")
    }

    pub fn from_provider_id(id: &str) -> Option<Self> {
        match id {
            PROVIDER_ZHIPU => Some(Provider::Zhipu),
            PROVIDER_KIMI => Some(Provider::Kimi),
            _ => None,
        }
    }
}

pub type CodingPlan = Provider;

#[derive(Debug, Clone)]
pub enum UsageInfo {
    Zhipu(ZhipuUsageInfo),
    Kimi(KimiUsageInfo),
}

impl UsageInfo {
    pub fn provider_id(&self) -> &str {
        match self {
            UsageInfo::Zhipu(_) => PROVIDER_ZHIPU,
            UsageInfo::Kimi(_) => PROVIDER_KIMI,
        }
    }

    pub fn provider(&self) -> Provider {
        match self {
            UsageInfo::Zhipu(_) => Provider::Zhipu,
            UsageInfo::Kimi(_) => Provider::Kimi,
        }
    }

    pub fn is_zhipu(&self) -> bool {
        matches!(self, UsageInfo::Zhipu(_))
    }

    pub fn is_kimi(&self) -> bool {
        matches!(self, UsageInfo::Kimi(_))
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
