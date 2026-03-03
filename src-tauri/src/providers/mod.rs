pub mod zhipu;
pub mod kimi;

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::provider::{Provider, ProviderRegistry};

pub static PROVIDERS: LazyLock<Vec<&'static dyn Provider>> = LazyLock::new(|| {
    inventory::iter::<ProviderRegistry>().into_iter().map(|r| r.0).collect()
});

pub static PROVIDER_MAP: LazyLock<HashMap<&'static str, &'static dyn Provider>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for provider in PROVIDERS.iter() {
        map.insert(provider.id(), *provider);
    }
    map
});

pub fn get_provider_by_id(id: &str) -> Option<&'static dyn Provider> {
    PROVIDER_MAP.get(id).copied()
}
