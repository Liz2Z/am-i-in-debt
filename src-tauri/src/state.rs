use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tauri::tray::TrayIcon;

use crate::provider::{UsageInfo, format_last_update_time};

pub struct AppState {
    pub usage_info: Arc<Mutex<Vec<Box<dyn UsageInfo>>>>,
    pub tray: Arc<Mutex<Option<TrayIcon>>>,
    pub last_update_time: Arc<Mutex<Option<i64>>>,
    pub exhausted_notified: Arc<Mutex<HashSet<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            usage_info: Arc::new(Mutex::new(Vec::new())),
            tray: Arc::new(Mutex::new(None)),
            last_update_time: Arc::new(Mutex::new(None)),
            exhausted_notified: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn set_usage(&self, usage_list: Vec<Box<dyn UsageInfo>>) {
        let mut info = self.usage_info.lock().unwrap();
        *info = usage_list;
    }

    pub fn with_usage<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Box<dyn UsageInfo>]) -> R,
    {
        let info = self.usage_info.lock().unwrap();
        f(&info)
    }

    pub fn update_usage_and_get<F, R>(&self, usage_list: Vec<Box<dyn UsageInfo>>, f: F) -> R
    where
        F: FnOnce(&[Box<dyn UsageInfo>]) -> R,
    {
        let mut info = self.usage_info.lock().unwrap();
        *info = usage_list;
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
}
