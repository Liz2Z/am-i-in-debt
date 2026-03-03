mod zhipu;
mod kimi;

pub use zhipu::fetch_zhipu_usage;
pub use kimi::fetch_kimi_usage;

use crate::error::Result;
use crate::models::{Provider, UsageInfo, ALL_PROVIDERS};
use std::future::Future;
use std::pin::Pin;
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::sync::LazyLock;

pub trait UsageFetcher: Send + Sync {
    fn fetch<'a>(&'a self, cookie_path: &'a Path) -> Pin<Box<dyn Future<Output = Result<UsageInfo>> + Send + 'a>>;
}

struct ZhipuFetcher;
struct KimiFetcher;

impl UsageFetcher for ZhipuFetcher {
    fn fetch<'a>(&'a self, cookie_path: &'a Path) -> Pin<Box<dyn Future<Output = Result<UsageInfo>> + Send + 'a>> {
        Box::pin(fetch_zhipu_usage(cookie_path))
    }
}

impl UsageFetcher for KimiFetcher {
    fn fetch<'a>(&'a self, cookie_path: &'a Path) -> Pin<Box<dyn Future<Output = Result<UsageInfo>> + Send + 'a>> {
        Box::pin(fetch_kimi_usage(cookie_path))
    }
}

pub static FETCHER_REGISTRY: LazyLock<HashMap<&'static str, Box<dyn UsageFetcher>>> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, Box<dyn UsageFetcher>> = HashMap::new();
    map.insert(Provider::ZHIPU.id, Box::new(ZhipuFetcher));
    map.insert(Provider::KIMI.id, Box::new(KimiFetcher));
    map
});

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

pub async fn fetch_usage_for_provider(provider: Provider) -> Option<UsageInfo> {
    let cookie_path = provider.cookie_path();
    if let Some(fetcher) = FETCHER_REGISTRY.get(provider.id) {
        fetcher.fetch(&cookie_path).await.ok()
    } else {
        None
    }
}

pub async fn fetch_all_usage() -> Vec<UsageInfo> {
    let mut usage_list = Vec::new();
    for provider in ALL_PROVIDERS.iter() {
        if let Some(info) = fetch_usage_for_provider(*provider).await {
            usage_list.push(info);
        }
    }
    usage_list
}
