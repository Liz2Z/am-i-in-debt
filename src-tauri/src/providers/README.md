# Providers 开发指南

本目录包含所有 provider 的实现。每个 provider 是一个自包含的模块，通过 `inventory` crate 实现自注册。

## 架构概述

```
providers/
├── mod.rs        # 收集所有 provider，通过 inventory 自动注册
├── zhipu.rs      # 智谱 provider 实现
├── kimi.rs       # Kimi provider 实现
└── README.md     # 本文档
```

## 添加新 Provider

只需创建一个新的 `.rs` 文件，无需修改其他任何代码。

### 步骤 1: 创建新文件

在 `providers/` 目录下创建新文件，如 `new_provider.rs`：

```rust
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::future::Future;

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, IsMenuItem, MenuItem, PredefinedMenuItem},
    AppHandle, Wry,
};

use crate::error::{AppError, Result};
use crate::provider::{format_progress_bar, Provider, ProviderRegistry, UsageInfo};

// ============== 常量定义 ==============

pub const NEW_PROVIDER_ID: &str = "new-provider-coding-plan";
pub const NEW_PROVIDER_DISPLAY_NAME: &str = "NewProvider";
pub const NEW_PROVIDER_LOGIN_ARG: &str = "new-provider";
pub const NEW_PROVIDER_AUTH_TOKEN: &str = "new-provider-token";

// ============== Provider 实现 ==============

pub struct NewProviderProvider;

impl Provider for NewProviderProvider {
    fn id(&self) -> &'static str { NEW_PROVIDER_ID }
    fn display_name(&self) -> &'static str { NEW_PROVIDER_DISPLAY_NAME }
    fn login_script_arg(&self) -> &'static str { NEW_PROVIDER_LOGIN_ARG }
    fn auth_token_name(&self) -> &'static str { NEW_PROVIDER_AUTH_TOKEN }

    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<Box<dyn UsageInfo>>> + Send + 'static>> {
        Box::pin(async move {
            // 1. 读取 cookies
            // 2. 调用 API 获取用量
            // 3. 解析响应
            // 4. 返回 Box::new(info) as Box<dyn UsageInfo>
            todo!("实现 API 调用逻辑")
        })
    }
}

// ============== 自注册 ==============

pub static NEW_PROVIDER: NewProviderProvider = NewProviderProvider;

inventory::submit!(ProviderRegistry(&NEW_PROVIDER));

// ============== UsageInfo 实现 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProviderUsageInfo {
    pub token_total: i64,
    pub token_used: i64,
    // ... 其他字段
}

impl UsageInfo for NewProviderUsageInfo {
    fn provider_id(&self) -> &'static str {
        NEW_PROVIDER_ID
    }

    fn is_token_exhausted(&self) -> bool {
        // 定义 token 耗尽的判断逻辑
        self.token_remaining <= 0
    }

    fn render_menu_items(
        &self,
        app: &AppHandle,
        is_selected: bool,
    ) -> Vec<Box<dyn IsMenuItem<Wry>>> {
        let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();

        // Provider 选择项
        items.push(Box::new(CheckMenuItem::with_id(
            app,
            format!("select-{}", NEW_PROVIDER_ID),
            format!("{} Coding Plan", NEW_PROVIDER_DISPLAY_NAME),
            true,
            is_selected,
            None::<&str>,
        ).unwrap()));

        // 用量显示项
        // ...

        items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
        items
    }

    fn clone_boxed(&self) -> Box<dyn UsageInfo> {
        Box::new(self.clone())
    }
}

// ============== 内部辅助函数 ==============

fn read_cookies(cookie_path: &Path) -> Result<Vec<serde_json::Value>> {
    if !cookie_path.exists() {
        return Err(AppError::Auth("未找到 cookies，请先登录".to_string()));
    }
    let cookie_content = std::fs::read_to_string(cookie_path)?;
    let cookies: Vec<serde_json::Value> = serde_json::from_str(&cookie_content)?;
    Ok(cookies)
}

fn find_cookie_value<'a>(cookies: &'a [serde_json::Value], name: &str) -> Option<&'a str> {
    cookies
        .iter()
        .find(|c| c["name"] == name)
        .and_then(|c| c["value"].as_str())
}
```

### 步骤 2: 在 mod.rs 中添加模块声明

编辑 `mod.rs`，添加一行：

```rust
pub mod new_provider;
```

就这样！无需修改任何其他文件。

## Trait 说明

### Provider Trait

```rust
pub trait Provider: Send + Sync + 'static {
    fn id(&self) -> &'static str;                    // 唯一标识，如 "zhipu-coding-plan"
    fn display_name(&self) -> &'static str;          // 显示名称，如 "智谱"
    fn login_script_arg(&self) -> &'static str;      // 登录脚本参数
    fn auth_token_name(&self) -> &'static str;       // cookie 中的 token 名称

    fn data_dir(&self) -> PathBuf;                   // 数据目录（自动实现）
    fn cookie_path(&self) -> PathBuf;                // cookie 文件路径（自动实现）
    fn settings_path(&self) -> PathBuf;              // 设置文件路径（自动实现）

    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<Box<dyn UsageInfo>>> + Send + 'static>>;
}
```

### UsageInfo Trait

```rust
pub trait UsageInfo: Send + Sync + 'static {
    fn provider_id(&self) -> &'static str;           // 返回对应的 provider id
    fn is_token_exhausted(&self) -> bool;            // 判断 token 是否耗尽
    fn render_menu_items(&self, app: &AppHandle, is_selected: bool) -> Vec<Box<dyn IsMenuItem<Wry>>>;
    fn clone_boxed(&self) -> Box<dyn UsageInfo>;     // 克隆方法
}
```

## 文件结构约定

每个 provider 文件应包含：

1. **常量定义** - ID、显示名称、登录参数、token 名称
2. **Provider struct** - 实现 `Provider` trait
3. **静态实例** - `pub static XXX: XxxProvider`
4. **inventory 注册** - `inventory::submit!(ProviderRegistry(&XXX));`
5. **UsageInfo struct** - 实现 `UsageInfo` trait
6. **辅助函数** - cookie 读取、API 调用等

## 登录脚本

登录脚本位于 `scripts/` 目录，命名约定：`login_<provider_login_arg>.sh`

例如 `NEW_PROVIDER_LOGIN_ARG = "new-provider"`，则脚本为 `scripts/login_new-provider.sh`。

登录脚本负责：

1. 打开浏览器到登录页面
2. 使用 Chrome DevTools Protocol 等待用户登录
3. 提取 cookies 并保存到 `~/.local/share/am-i-in-debt/<provider_id>/cookies.json`
