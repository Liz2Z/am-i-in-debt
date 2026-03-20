use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tauri::menu::{CheckMenuItem, MenuItem};
use tauri::tray::TrayIcon;
use tauri::Wry;

use crate::error::AppError;
use crate::provider::{format_last_update_time, UsageInfo};

#[derive(Debug, Clone, PartialEq)]
pub enum FetchStatus {
    Ok,
    HttpError(String),
    AuthError,
}

pub struct FetchResult {
    pub provider_id: &'static str,
    pub result: std::result::Result<Box<dyn UsageInfo>, AppError>,
}

pub struct MenuHandles {
    pub items: HashMap<String, MenuItem<Wry>>,
    pub checks: HashMap<String, CheckMenuItem<Wry>>,
}

impl MenuHandles {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            checks: HashMap::new(),
        }
    }
}

pub struct AppState {
    pub usage_info: Arc<Mutex<Vec<Box<dyn UsageInfo>>>>,
    pub tray: Arc<Mutex<Option<TrayIcon>>>,
    pub last_update_time: Arc<Mutex<Option<i64>>>,
    pub exhausted_notified: Arc<Mutex<HashSet<String>>>,
    pub fetch_status: Arc<Mutex<HashMap<String, FetchStatus>>>,
    pub refresh_interval_secs: Arc<Mutex<u64>>,
    pub menu_handles: Arc<Mutex<Option<MenuHandles>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            usage_info: Arc::new(Mutex::new(Vec::new())),
            tray: Arc::new(Mutex::new(None)),
            last_update_time: Arc::new(Mutex::new(None)),
            exhausted_notified: Arc::new(Mutex::new(HashSet::new())),
            fetch_status: Arc::new(Mutex::new(HashMap::new())),
            refresh_interval_secs: Arc::new(Mutex::new(300)),
            menu_handles: Arc::new(Mutex::new(None)),
        }
    }

    /// 增量合并拉取结果：Auth 错误移除旧数据，Http 错误保留旧数据
    pub fn merge_usage(&self, results: Vec<FetchResult>) {
        let mut info = self.usage_info.lock().unwrap();
        let mut status = self.fetch_status.lock().unwrap();
        for r in results {
            match r.result {
                Ok(new_data) => {
                    info.retain(|u| u.provider_id() != r.provider_id);
                    info.push(new_data);
                    status.insert(r.provider_id.to_string(), FetchStatus::Ok);
                }
                Err(AppError::Auth(_)) => {
                    info.retain(|u| u.provider_id() != r.provider_id);
                    status.insert(r.provider_id.to_string(), FetchStatus::AuthError);
                }
                Err(e) => {
                    // 保留旧数据，仅更新状态标记
                    log::warn!("{} 拉取失败（保留旧数据）: {}", r.provider_id, e);
                    status.insert(
                        r.provider_id.to_string(),
                        FetchStatus::HttpError(e.to_string()),
                    );
                }
            }
        }
    }

    pub fn with_usage<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Box<dyn UsageInfo>]) -> R,
    {
        let info = self.usage_info.lock().unwrap();
        f(&info)
    }

    pub fn update_time(&self) {
        use chrono::Utc;
        let now = Utc::now();
        let timestamp_ms = now.timestamp_millis();
        *self.last_update_time.lock().unwrap() = Some(timestamp_ms);
    }

    pub fn get_update_time_suffix(&self) -> String {
        let last_update = self.last_update_time.lock().unwrap();
        if let Some(ts) = *last_update {
            format!(" ({})", format_last_update_time(ts))
        } else {
            "".to_string()
        }
    }

    pub fn set_tray(&self, tray: TrayIcon) {
        *self.tray.lock().unwrap() = Some(tray);
    }

    pub fn get_tray(&self) -> Option<TrayIcon> {
        self.tray.lock().unwrap().clone()
    }

    pub fn should_notify_exhausted(&self, provider_id: &str) -> bool {
        let mut notified = self.exhausted_notified.lock().unwrap();
        if notified.contains(provider_id) {
            false
        } else {
            notified.insert(provider_id.to_string());
            true
        }
    }

    pub fn clear_exhausted_notification(&self, provider_id: &str) {
        let mut notified = self.exhausted_notified.lock().unwrap();
        notified.remove(provider_id);
    }

    pub fn get_refresh_interval(&self) -> u64 {
        *self.refresh_interval_secs.lock().unwrap()
    }

    pub fn set_refresh_interval(&self, secs: u64) {
        *self.refresh_interval_secs.lock().unwrap() = secs;
    }

    pub fn store_menu_handles(&self, handles: MenuHandles) {
        *self.menu_handles.lock().unwrap() = Some(handles);
    }

    pub fn get_fetch_status(&self, provider_id: &str) -> Option<FetchStatus> {
        self.fetch_status.lock().unwrap().get(provider_id).cloned()
    }
}
