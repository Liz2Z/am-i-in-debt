use std::sync::{Arc, Mutex};
use tauri::tray::TrayIcon;

use crate::providers::{UsageInfo, format_last_update_time};

pub struct AppState {
    pub usage_info: Arc<Mutex<Vec<UsageInfo>>>,
    pub tray: Arc<Mutex<Option<TrayIcon>>>,
    pub last_update_time: Arc<Mutex<Option<i64>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            usage_info: Arc::new(Mutex::new(Vec::new())),
            tray: Arc::new(Mutex::new(None)),
            last_update_time: Arc::new(Mutex::new(None)),
        }
    }

    pub fn update_usage(&self, usage_list: Vec<UsageInfo>) {
        let mut info = self.usage_info.lock().unwrap();
        *info = usage_list;
    }

    pub fn get_usage(&self) -> Vec<UsageInfo> {
        self.usage_info.lock().unwrap().clone()
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
}
