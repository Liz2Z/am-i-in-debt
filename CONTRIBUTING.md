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

应用使用 **Provider 模式**，添加新 provider 只需创建一个文件：

1. 创建 `src/providers/newplan.rs`，实现 `Provider` trait
2. 在文件末尾使用 `inventory::submit!` 自注册
3. 在 `providers/mod.rs` 添加 `pub mod newplan;`
4. 在 `get-cookies-script/src/` 创建登录脚本（如 `newplan.ts`）
5. 更新文档

#### Provider Trait 接口

```rust
pub trait Provider: Send + Sync + 'static {
    fn id(&self) -> &'static str;              // provider 唯一标识
    fn display_name(&self) -> &'static str;    // 显示名称
    fn login_script_arg(&self) -> &'static str; // 登录脚本参数
    fn auth_token_name(&self) -> &'static str;  // 认证 token 名称
    
    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<UsageInfo>> + Send + 'static>>;
    fn render_menu_items(&self, app: &AppHandle, usage: &UsageInfo, is_selected: bool) -> Vec<Box<dyn IsMenuItem<Wry>>>;
}
```

#### 示例 Provider 实现

```rust
use crate::provider::{Provider, ProviderRegistry, UsageInfo};
use crate::error::Result;

pub const NEW_PROVIDER_ID: &str = "new-provider-id";
pub const NEW_PROVIDER_DISPLAY_NAME: &str = "新供应商";
pub const NEW_PROVIDER_LOGIN_ARG: &str = "new-provider";
pub const NEW_PROVIDER_AUTH_TOKEN: &str = "auth_token";

pub struct NewProvider;

impl Provider for NewProvider {
    fn id(&self) -> &'static str { NEW_PROVIDER_ID }
    fn display_name(&self) -> &'static str { NEW_PROVIDER_DISPLAY_NAME }
    fn login_script_arg(&self) -> &'static str { NEW_PROVIDER_LOGIN_ARG }
    fn auth_token_name(&self) -> &'static str { NEW_PROVIDER_AUTH_TOKEN }
    
    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<UsageInfo>> + Send + 'static>> {
        // 实现 API 调用
    }
    
    fn render_menu_items(&self, app: &AppHandle, usage: &UsageInfo, is_selected: bool) -> Vec<Box<dyn IsMenuItem<Wry>>> {
        // 实现菜单渲染
    }
}

pub static NEW_PROVIDER: NewProvider = NewProvider;

// 自注册
inventory::submit!(ProviderRegistry(&NEW_PROVIDER));
```

## 许可证

通过提交代码，你同意你的贡献将按照 MIT 许可证授权。
