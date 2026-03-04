# Contributing to Am I In Debt

感谢你考虑为 Am I In Debt 做贡献！

## 如何贡献

### 报告 Bug

如果你发现了 bug，请通过 [GitHub Issues](../../issues) 提交报告。请包含：

- 操作系统版本
- 应用版本
- 复现步骤
- 预期行为和实际行为
- 相关日志（如有）

### 提交新功能

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/your-username/am-i-in-debt.git
cd am-i-in-debt

# 安装依赖
bun install

# 构建 sidecar
bun run build:sidecar

# 开发模式运行
bun run tauri:dev
```

### 代码风格

- Rust 代码遵循 `cargo fmt` 格式
- TypeScript 代码使用项目中的 ESLint 配置
- 提交信息遵循 [Conventional Commits](https://www.conventionalcommits.org/)

### 添加新的 Coding Plan 支持

应用使用 **Provider 模式**，添加新 provider 只需创建一个文件。详细步骤请参考 [providers/README.md](../src-tauri/src/providers/README.md)。

简要步骤：

1. 在 `src-tauri/src/providers/` 创建新文件（如 `newplan.rs`）
2. 实现 `Provider` trait
3. 实现 `UsageInfo` trait
4. 使用 `inventory::submit!` 注册
5. 在 `mod.rs` 添加 `pub mod newplan;`
6. 在 `get-cookies-script/src/` 创建登录脚本（如 `newplan.ts`）

**无需修改其他任何文件！**

#### 核心 Traits

```rust
// Provider Trait
pub trait Provider: Send + Sync + 'static {
    fn id(&self) -> &'static str;              // provider 唯一标识
    fn display_name(&self) -> &'static str;    // 显示名称
    fn login_script_arg(&self) -> &'static str; // 登录脚本参数
    fn auth_token_name(&self) -> &'static str;  // 认证 token 名称

    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<Box<dyn UsageInfo>>> + Send + 'static>>;
}

// UsageInfo Trait
pub trait UsageInfo: Send + Sync + 'static {
    fn provider_id(&self) -> &'static str;     // 返回对应的 provider id
    fn is_token_exhausted(&self) -> bool;      // 判断 token 是否耗尽
    fn render_menu_items(&self, app: &AppHandle, is_selected: bool) -> Vec<Box<dyn IsMenuItem<Wry>>>;
    fn clone_boxed(&self) -> Box<dyn UsageInfo>;
}
```

#### 示例 Provider 实现

```rust
use std::path::PathBuf;
use std::pin::Pin;
use std::future::Future;

use tauri::{
    menu::{CheckMenuItem, IsMenuItem, MenuItem, PredefinedMenuItem},
    AppHandle, Wry,
};

use crate::error::{AppError, Result};
use crate::provider::{Provider, ProviderRegistry, UsageInfo};

pub const NEW_PROVIDER_ID: &str = "new-provider-coding-plan";
pub const NEW_PROVIDER_DISPLAY_NAME: &str = "NewProvider";
pub const NEW_PROVIDER_LOGIN_ARG: &str = "new-provider";
pub const NEW_PROVIDER_AUTH_TOKEN: &str = "auth_token";

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
            // 3. 返回 Box::new(info) as Box<dyn UsageInfo>
            todo!()
        })
    }
}

pub static NEW_PROVIDER: NewProviderProvider = NewProviderProvider;

inventory::submit!(ProviderRegistry(&NEW_PROVIDER));

#[derive(Debug, Clone)]
pub struct NewProviderUsageInfo {
    pub token_remaining: i64,
    // ... 其他字段
}

impl UsageInfo for NewProviderUsageInfo {
    fn provider_id(&self) -> &'static str {
        NEW_PROVIDER_ID
    }

    fn is_token_exhausted(&self) -> bool {
        self.token_remaining <= 0
    }

    fn render_menu_items(&self, app: &AppHandle, is_selected: bool) -> Vec<Box<dyn IsMenuItem<Wry>>> {
        let mut items = Vec::new();

        items.push(Box::new(CheckMenuItem::with_id(
            app,
            format!("select-{}", NEW_PROVIDER_ID),
            format!("{} Coding Plan", NEW_PROVIDER_DISPLAY_NAME),
            true,
            is_selected,
            None::<&str>,
        ).unwrap()));

        // 添加更多菜单项...

        items
    }

    fn clone_boxed(&self) -> Box<dyn UsageInfo> {
        Box::new(self.clone())
    }
}
```

## 许可证

通过提交代码，你同意你的贡献将按照 MIT 许可证授权。
