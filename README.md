# Coding Plan Usage Monitor

一个 macOS 状态栏应用，用于监控多个 Coding Plan（智谱、Kimi）的使用情况。

## 功能特性

- 🍎 **纯状态栏应用**：不在 Dock 栏显示图标
- 📊 **多平台支持**：支持智谱 Coding Plan 和 Kimi Coding Plan
- 🔐 **自动化登录**：使用 Chrome DevTools Protocol 自动获取 cookies
- 📈 **使用情况展示**：显示已用/总计/剩余 tokens、进度条、重置时间
- 🔄 **自动刷新**：每 30 秒自动更新数据
- 💾 **XDG 规范存储**：数据存储在 `~/.local/share/coding-plan-usage/`

## 技术栈

- **前端框架**：Tauri 2.x
- **后端语言**：Rust
- **脚本语言**：TypeScript (Bun runtime)
- **浏览器自动化**：Chrome DevTools Protocol (`chrome-remote-interface`)

## 项目结构

```
coding-plan-usage/
├── src-tauri/
│   ├── src/
│   │   └── main.rs          # 主应用代码 (Rust)
│   ├── bin/
│   │   ├── get-zhipu-cookies # 智谱 cookie 获取脚本 (编译后)
│   │   └── get-kimi-cookies  # Kimi cookie 获取脚本 (编译后)
│   ├── icons/                # 托盘图标
│   └── Cargo.toml
├── get-cookies-script/
│   ├── src/
│   │   ├── index.ts          # 智谱 cookie 获取脚本源码
│   │   └── kimi.ts           # Kimi cookie 获取脚本源码
│   └── package.json
└── README.md
```

## 数据存储

按照 XDG Base Directory Specification，数据存储在：

```
~/.local/share/coding-plan-usage/
├── zhipu-coding-plan/
│   └── cookies.json
└── kimi-coding-plan/
    └── cookies.json
```

## API 接口

### 智谱 Coding Plan

- **登录页面**: `https://bigmodel.cn/usercenter/glm-coding/usage`
- **Cookie 名称**: `bigmodel_token_production`
- **使用情况接口**: `GET https://bigmodel.cn/api/monitor/usage/quota/limit`
- **认证方式**: `authorization: <token>`

### 智谱 API 响应格式

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "limits": [
      {
        "type": "TIME_LIMIT",
        "unit": 5, // 月
        "number": 1, // 代表“每个月的 MCP 使用额度“
        "usage": 100, // 已使用 100 次
        "currentValue": 101,
        "remaining": 0,
        "percentage": 100,
        "nextResetTime": 1772764847997, // 额度重置时间的时间戳
        "usageDetails": [
          {
            "modelCode": "search-prime", // 搜索 mcp
            "usage": 79 // 使用了 79 次
          },
          {
            "modelCode": "web-reader", // 网页读取 mcp
            "usage": 22
          },
          {
            "modelCode": "zread",
            "usage": 0
          }
        ]
      },
      {
        "type": "TOKENS_LIMIT",
        "unit": 3, // 小时
        "number": 5, // 代表“每 5 小时使用额度“
        "percentage": 69, // 使用额度比例
        "nextResetTime": 1772473296701 // 额度重置时间的时间戳
      }
    ],
    "level": "lite"
  },
  "success": true
}
```

### 智谱 信息展示

```text
Token 额度（每 x 小时）
[进度条] 百分比数值
重置: 26-01-02 13:00:00
------------------------
MCP 额度（每月）
[进度条] 百分比数值
搜索: xx | 网页: xx | zreader: xxx
重置: 26-01-02 13: 00: 00
```

### Kimi Coding Plan

- **登录页面**: `https://www.kimi.com/code/console`
- **Cookie 名称**: `kimi-auth` (HttpOnly)
- **使用情况接口**: `POST https://www.kimi.com/apiv2/kimi.gateway.billing.v1.BillingService/GetUsages`
- **认证方式**: `authorization: Bearer <token>`
- **请求体**: `{"scope": ["FEATURE_CODING"]}`

### Kimi API 响应格式

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
          "window": {
            "duration": 300,
            "timeUnit": "TIME_UNIT_MINUTE"
          },
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

**说明**：

- `detail`：本周使用量
- `limits[0].detail`：5 小时窗口使用量（应用显示此数据）

### KIMI 信息展示

```text
Token 额度（每 {limits[0].window.duration/60} 小时）
[进度条] {limits[0].detail.limit - limits[0].detail.remaining} %
重置: {limits[0].detail.resetTime} 
------------------------
Token 额度（每周）
[进度条] {detail.used}%
重置: {detail.resetTime}
```

## 登录流程

### 智谱 Coding Plan 登录

1. 启动 Chrome（临时用户数据目录，端口 9222）
2. 打开 `https://bigmodel.cn/usercenter/glm-coding/usage`
3. 等待用户完成登录
4. 检测 URL 跳转到 usage 页面
5. 获取所有 cookies 并保存

### Kimi Coding Plan 登录

1. 启动 Chrome（临时用户数据目录，端口 9223）
2. 打开 `https://www.kimi.com/code/console`
3. 清理可能占用端口的进程
4. 等待用户完成登录
5. **每 3 秒检查一次**：
   - 使用 CDP 获取 `kimi-auth` cookie（包括 HttpOnly）
   - 调用 usage 接口验证 token
   - 直到接口返回有效数据
6. 保存 cookies

### CDP Cookie 获取

```typescript
// 获取特定域名的 cookies（包括 HttpOnly）
const cookies = await Network.getCookies({
  urls: ["https://www.kimi.com/"],
});
```

## 开发指南

### 环境要求

- macOS
- Rust (1.70+)
- Bun
- Chrome 浏览器

### 构建步骤

1. **编译 cookie 获取脚本**：

```bash
cd get-cookies-script
bun run build
```

2. **运行开发版本**：

```bash
cd src-tauri
cargo run
```

3. **构建发布版本**：

```bash
cd src-tauri
cargo build --release
```

### 添加新的 Coding Plan

1. 在 `get-cookies-script/src/` 创建新的登录脚本
2. 在 `src-tauri/src/main.rs` 添加新的 `CodingPlan` 枚举值
3. 实现对应的 API 调用逻辑
4. 更新 `package.json` 的 build 脚本

## 配置文件

### Tauri 配置 (`src-tauri/tauri.conf.json`)

```json
{
  "app": {
    "windows": [],
    "macOSPrivateApi": true
  }
}
```

### Cargo.toml 关键依赖

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-ico", "image-png", "macos-private-api"] }
serde = { version = "1", features = ["derive"] }
reqwest = { version = "0.12", features = ["json"] }
chrono = "0.4"
```

## 故障排除

### Chrome 启动超时

- 检查是否有其他进程占用了调试端口 (9222/9223)
- 清理临时 Chrome 进程：`pkill -f chrome-debug`

### Cookie 获取失败

- 确认 Chrome 远程调试端口已开启
- 检查 cookie 名称是否正确（注意大小写）
- HttpOnly cookie 需要使用 `Network.getCookies()` 获取

### Token 验证失败

- 检查 API 接口是否有更新
- 确认认证方式（Bearer / 直接 token）
- 查看响应中的 `code` 字段

## 许可证

MIT
