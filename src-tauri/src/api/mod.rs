mod zhipu;
mod kimi;

pub use zhipu::fetch_zhipu_usage;
pub use kimi::fetch_kimi_usage;

use crate::error::Result;
use crate::models::Provider;
use std::fs;
use std::path::Path;

pub trait UsageFetcher {
    fn provider(&self) -> Provider;
    fn fetch(&self, cookie_path: &Path) -> impl std::future::Future<Output = Result<crate::models::UsageInfo>> + Send;
}

pub fn read_cookies(cookie_path: &Path) -> Result<Vec<serde_json::Value>> {
    if !cookie_path.exists() {
        return Err(crate::error::AppError::Auth("未找到 cookies，请先登录".to_string()));
    }
    let cookie_content = fs::read_to_string(cookie_path)?;
    let cookies: Vec<serde_json::Value> = serde_json::from_str(&cookie_content)?;
    Ok(cookies)
}

pub fn find_cookie_value<'a>(cookies: &'a [serde_json::Value], name: &str) -> Option<&'a str> {
    cookies
        .iter()
        .find(|c| c["name"] == name)
        .and_then(|c| c["value"].as_str())
}

pub async fn fetch_usage_for_provider(provider: Provider) -> Option<crate::models::UsageInfo> {
    let cookie_path = provider.cookie_path();
    match provider {
        Provider::Zhipu => fetch_zhipu_usage(&cookie_path).await.ok(),
        Provider::Kimi => fetch_kimi_usage(&cookie_path).await.ok(),
    }
}
