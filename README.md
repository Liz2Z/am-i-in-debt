# Am I In Debt

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform: macOS](https://img.shields.io/badge/Platform-macOS-lightgrey.svg)](https://www.apple.com/macos)

一个 macOS 状态栏应用，用于监控多个 Coding Plan（智谱、Kimi）的使用情况。

![Screenshot](image.png)

## 功能特性

- 🍎 **纯状态栏应用**：不在 Dock 栏显示图标
- 📊 **多平台支持**：支持智谱 Coding Plan 和 Kimi Coding Plan
- 🔐 **自动化登录**：使用 Chrome DevTools Protocol 自动获取 cookies
- 📈 **使用情况展示**：显示已用/总计/剩余 tokens、进度条、重置时间
- 🔄 **自动刷新**：每 30 秒自动更新数据，支持手动刷新
- 💾 **XDG 规范存储**：数据存储在 `~/.local/share/am-i-in-debt/`

## 支持的平台

| 平台             | 功能                                   |
| ---------------- | -------------------------------------- |
| 智谱 Coding Plan | Token 额度、MCP 额度（搜索/网页/阅读） |
| Kimi Coding Plan | 小时额度、周额度                       |

## 快速开始

### 环境要求

- macOS
- Rust (1.70+)
- Bun
- Chrome 浏览器

### 开发命令

```bash
# 安装依赖
bun install

# 构建 sidecar
bun run build:sidecar

# 开发模式运行
bun run tauri:dev

# 构建发布版本
bun run tauri:build
```

### 构建产物

构建完成后，产物位于：

- **App**: `src-tauri/target/release/bundle/macos/Am I In Debt.app`
- **DMG**: `src-tauri/target/release/bundle/dmg/Am I In Debt_1.0.0_aarch64.dmg`

## 故障排除

### Chrome 启动超时

- 检查是否有其他进程占用了调试端口 (9222/9223)
- 清理临时 Chrome 进程：`pkill -f chrome-debug`

### Cookie 获取失败

- 确认 Chrome 远程调试端口已开启
- 检查 cookie 名称是否正确（注意大小写）
- HttpOnly cookie 需要使用 `Network.getCookies()` 获取

## 文档

- [技术文档](docs/TECHNICAL.md) - API 接口、登录流程、架构设计
- [贡献指南](CONTRIBUTING.md) - 如何添加新的 Provider
- [开发规范](docs/RULES.md) - 代码风格和日志规范

## 许可证

MIT
