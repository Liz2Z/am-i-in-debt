# 技术文档

## 项目结构

```
am-i-in-debt/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # 应用入口
│   │   ├── lib.rs            # 库导出
│   │   ├── provider.rs       # Provider trait, UsageInfo trait 和公共工具函数
│   │   ├── providers/        # Provider 实现（自包含模块）
│   │   │   ├── mod.rs        # 从 inventory 收集注册的 provider
│   │   │   ├── zhipu.rs      # 智谱 provider（完全自包含）
│   │   │   ├── kimi.rs       # Kimi provider（完全自包含）
│   │   │   └── README.md     # Provider 开发指南
│   │   ├── menu.rs           # 菜单逻辑
│   │   ├── state.rs          # 状态管理
│   │   ├── login.rs          # 登录逻辑
│   │   ├── provider_switch.rs # Provider 切换逻辑
│   │   └── error.rs          # 统一错误类型
│   ├── bin/
│   │   └── get-cookies       # 统一的 cookie 获取脚本
│   ├── icons/                # 应用图标
│   └── Cargo.toml
├── get-cookies-script/
│   ├── src/
│   │   ├── index.ts          # 统一入口（根据参数调用）
│   │   ├── chrome.ts         # 公共 Chrome 启动逻辑
│   │   ├── zhipu.ts          # 智谱登录逻辑
│   │   └── kimi.ts           # Kimi 登录逻辑
│   ├── tsconfig.json
│   └── package.json
└── README.md
```

## 数据存储

按照 XDG Base Directory Specification，数据存储在：

```
~/.local/share/am-i-in-debt/
├── zhipu-coding-plan/
│   ├── cookies.json
│   └── settings.json         # Claude 配置（可选）
├── kimi-coding-plan/
│   ├── cookies.json
│   └── settings.json         # Claude 配置（可选）
└── state.json                # 当前选中的 provider
```

## API 接口

### 智谱 Coding Plan

- **登录页面**: `https://bigmodel.cn/usercenter/glm-coding/usage`
- **Cookie 名称**: `bigmodel_token_production`
- **使用情况接口**: `GET https://bigmodel.cn/api/monitor/usage/quota/limit`
- **认证方式**: `authorization: <token>`

#### API 响应格式

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "limits": [
      {
        "type": "TIME_LIMIT",
        "unit": 5,
        "number": 1,
        "usage": 100,
        "remaining": 0,
        "percentage": 100,
        "nextResetTime": 1772764847997,
        "usageDetails": [
          { "modelCode": "search-prime", "usage": 79 },
          { "modelCode": "web-reader", "usage": 22 },
          { "modelCode": "zread", "usage": 0 }
        ]
      },
      {
        "type": "TOKENS_LIMIT",
        "unit": 3,
        "number": 5,
        "percentage": 69,
        "nextResetTime": 1772473296701
      }
    ],
    "level": "lite"
  },
  "success": true
}
```

#### 信息展示

```text
Token 额度（每 x 小时）
[进度条] 百分比数值
重置: 26-01-02 13:00:00
-------------------------
MCP 额度（每月）
[进度条] 百分比数值
搜索: xx | 网页: xx | 阅读: xxx
重置: 26-01-02 13:00:00
```

### Kimi Coding Plan

- **登录页面**: `https://www.kimi.com/code/console`
- **Cookie 名称**: `kimi-auth` (HttpOnly)
- **使用情况接口**: `POST https://www.kimi.com/apiv2/kimi.gateway.billing.v1.BillingService/GetUsages`
- **认证方式**: `authorization: Bearer <token>`
- **请求体**: `{"scope": ["FEATURE_CODING"]}`

#### API 响应格式

```json
{
  "usages": [
    {
      "scope": "FEATURE_CODING",
      "detail": {
        "limit": "100",
        "used": "70",
        "remaining": "30",
        "resetTime": "2026-03-07T09:20:59.199525Z"
      },
      "limits": [
        {
          "window": { "duration": 300, "timeUnit": "TIME_UNIT_MINUTE" },
          "detail": {
            "limit": "100",
            "remaining": "100",
            "resetTime": "2026-03-02T19:20:59.199525Z"
          }
        }
      ]
    }
  ]
}
```

#### 信息展示

```text
Token 额度（每 5 小时）
[进度条] 使用百分比
重置: 重置时间
-------------------------
Token 额度（每周）
[进度条] 使用百分比
重置: 重置时间
```

## 登录流程

### 统一登录脚本

应用使用单一 sidecar 二进制文件，通过参数区分平台：

```bash
# 智谱登录
get-cookies zhipu

# Kimi 登录
get-cookies kimi
```

### 智谱 Coding Plan 登录

1. 启动 Chrome（临时用户数据目录，端口 9222）
2. 打开 `https://bigmodel.cn/usercenter/glm-coding/usage`
3. 等待用户完成登录
4. 检测 URL 跳转到 usage 页面
5. 获取所有 cookies 并保存

### Kimi Coding Plan 登录

1. 启动 Chrome（临时用户数据目录，端口 9223）
2. 打开 `https://www.kimi.com/code/console`
3. 等待用户完成登录
4. 每 3 秒检查一次：
   - 使用 CDP 获取 `kimi-auth` cookie（包括 HttpOnly）
   - 调用 usage 接口验证 token
   - 直到接口返回有效数据
5. 保存 cookies

## Provider 架构

应用使用 **Provider 模式** 实现多平台支持，每个 Provider 是一个完全自包含的模块：

- `provider.rs` - 定义 `Provider` trait、`UsageInfo` trait 和公共工具函数
- `providers/*.rs` - 每个 provider 实现所有逻辑（API、数据模型、菜单渲染）
- 使用 `inventory` crate 实现 provider 自注册

### 核心 Traits

#### Provider Trait

```rust
pub trait Provider: Send + Sync + 'static {
    fn id(&self) -> &'static str;              // provider 唯一标识
    fn display_name(&self) -> &'static str;    // 显示名称
    fn login_script_arg(&self) -> &'static str; // 登录脚本参数
    fn auth_token_name(&self) -> &'static str;  // 认证 token 名称

    // 以下方法有默认实现
    fn data_dir(&self) -> PathBuf;             // 数据目录
    fn cookie_path(&self) -> PathBuf;          // cookie 文件路径
    fn settings_path(&self) -> PathBuf;        // 设置文件路径

    // 必须实现
    fn fetch_usage(&self, cookie_path: PathBuf) -> Pin<Box<dyn Future<Output = Result<Box<dyn UsageInfo>>> + Send + 'static>>;
}
```

#### UsageInfo Trait

```rust
pub trait UsageInfo: Send + Sync + 'static {
    fn provider_id(&self) -> &'static str;     // 返回对应的 provider id
    fn is_token_exhausted(&self) -> bool;      // 判断 token 是否耗尽
    fn render_menu_items(&self, app: &AppHandle, is_selected: bool) -> Vec<Box<dyn IsMenuItem<Wry>>>;
    fn clone_boxed(&self) -> Box<dyn UsageInfo>; // 克隆方法
}
```

### 自注册机制

使用 `inventory` crate 实现编译期自注册：

```rust
// provider.rs
pub struct ProviderRegistry(pub &'static dyn Provider);
inventory::collect!(ProviderRegistry);

// providers/zhipu.rs
pub static ZHIPU: ZhipuProvider = ZhipuProvider;
inventory::submit!(ProviderRegistry(&ZHIPU));

// providers/mod.rs
pub static PROVIDERS: LazyLock<Vec<&'static dyn Provider>> = LazyLock::new(|| {
    inventory::iter::<ProviderRegistry>().into_iter().map(|r| r.0).collect()
});
```

### 添加新 Provider

详细步骤请参考 [providers/README.md](../src-tauri/src/providers/README.md)。

简要步骤：

1. 在 `providers/` 目录创建新文件（如 `new_provider.rs`）
2. 实现 `Provider` trait
3. 实现 `UsageInfo` trait
4. 使用 `inventory::submit!` 注册
5. 在 `mod.rs` 添加 `pub mod new_provider;`

**无需修改任何其他文件！**

## Token 耗尽通知

当当前选中的 provider token 耗尽时，应用会发送 macOS 通知：

- 只在首次耗尽时通知一次
- 当 token 恢复后重置通知状态
- 下次耗尽时会再次通知

通知逻辑在 `main.rs` 的 `check_exhausted_notification` 函数中实现。
