use crate::error::Result;
use crate::models::{Provider, get_provider_by_id};
use log::{error, info};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn get_claude_settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude/settings.json")
}

fn get_provider_settings_path(provider: Provider) -> PathBuf {
    provider.data_dir().join("settings.json")
}

fn get_app_state_path() -> PathBuf {
    crate::models::get_app_data_dir().join("state.json")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSelectionState {
    pub selected_provider: Option<String>,
}

impl Default for AppSelectionState {
    fn default() -> Self {
        Self {
            selected_provider: None,
        }
    }
}

pub fn load_selection_state() -> AppSelectionState {
    let path = get_app_state_path();
    if !path.exists() {
        return AppSelectionState::default();
    }
    
    match fs::read_to_string(&path) {
        Ok(content) => {
            match serde_json::from_str(&content) {
                Ok(state) => state,
                Err(e) => {
                    error!("解析状态文件失败: {}", e);
                    AppSelectionState::default()
                }
            }
        }
        Err(e) => {
            error!("读取状态文件失败: {}", e);
            AppSelectionState::default()
        }
    }
}

pub fn save_selection_state(state: &AppSelectionState) -> Result<()> {
    let path = get_app_state_path();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    
    let content = serde_json::to_string_pretty(state)?;
    fs::write(&path, content)?;
    info!("保存选中状态: {:?}", state.selected_provider);
    Ok(())
}

pub fn merge_settings(provider: Provider) -> Result<()> {
    let provider_settings_path = get_provider_settings_path(provider);
    
    if provider_settings_path.exists() {
        let claude_settings_path = get_claude_settings_path();
        
        let provider_content = fs::read_to_string(&provider_settings_path)?;
        let provider_settings: Value = serde_json::from_str(&provider_content)?;
        
        let claude_settings: Value = if claude_settings_path.exists() {
            let claude_content = fs::read_to_string(&claude_settings_path)?;
            serde_json::from_str(&claude_content)?
        } else {
            serde_json::json!({})
        };
        
        let merged = merge_json(&claude_settings, &provider_settings);
        
        let merged_content = serde_json::to_string_pretty(&merged)?;
        fs::write(&claude_settings_path, merged_content)?;
        
        info!("成功合并 {} 的配置到 ~/.claude/settings.json", provider.display_name);
    } else {
        info!("{} 的 settings.json 不存在，仅更新选中状态", provider.display_name);
    }
    
    let mut state = load_selection_state();
    state.selected_provider = Some(provider.id.to_string());
    save_selection_state(&state)?;
    
    Ok(())
}

fn merge_json(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            let mut result = base_map.clone();
            for (key, value) in overlay_map {
                if let Some(base_value) = result.get(key) {
                    result.insert(key.clone(), merge_json(base_value, value));
                } else {
                    result.insert(key.clone(), value.clone());
                }
            }
            Value::Object(result)
        }
        _ => overlay.clone(),
    }
}

pub fn get_current_selected_provider() -> Option<Provider> {
    let state = load_selection_state();
    info!("当前选中状态: {:?}", state.selected_provider);
    get_provider_by_id(state.selected_provider.as_deref()?)
}
