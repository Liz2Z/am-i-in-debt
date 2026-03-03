# 项目 Review 报告

> **项目**: Coding Plan Usage Monitor  
> **技术栈**: Tauri 2.x + Rust + TypeScript (Bun)  
> **平台**: macOS 状态栏应用  
> **审查日期**: 2026-03-03

---

## 项目概述

这是一个 macOS 状态栏应用，用于监控智谱和 Kimi 的 Coding Plan 使用情况。

---

## 🔴 架构问题

### 1. 单文件过大，缺少模块化

**文件**: `src-tauri/src/main.rs` (915 行)

建议重构为:
```
src-tauri/src/
├── main.rs        # 入口
├── lib.rs         # 库导出
├── api/           # API 客户端
├── models/        # 数据结构
├── menu.rs        # 菜单逻辑
└── state.rs       # 状态管理
```

### 2. lib.rs 几乎为空

应该将核心逻辑放在 lib.rs 中，便于测试和复用。

### 3. 状态管理设计问题

多层 `Arc<Mutex>` 嵌套，容易死锁。

### 4. 缺少抽象层

`fetch_zhipu_usage` 和 `fetch_kimi_usage` 有大量重复代码，应抽象为 trait。

---

## 🟠 工程问题

### 1. 项目命名不一致

| 位置 | 名称 |
|------|------|
| package.json | `am-i-in-debt` |
| Cargo.toml | `am-i-in-debt` |
| README 标题 | `Coding Plan Usage Monitor` |

### 2. tauri.conf.json 配置错误

`externalBin` 配置与实际文件不匹配。

### 3. 前端配置多余

vite.config.ts 和 tsconfig.json 配置了不存在的 src 目录。

### 4. 二进制文件被提交 (180MB+)

应添加到 .gitignore。

### 5. 未使用的依赖

`dirs` 和 `tauri-plugin-shell` 未使用。

### 6. 缺少测试和 CI/CD

---

## 🟡 代码质量问题

1. **硬编码配置** - 端口和 API URL 硬编码
2. **错误处理不一致** - 混用 String 和 unwrap
3. **潜在死锁风险** - 多次获取锁
4. **刷新功能未实现** - 按钮事件为空

---

## 🔵 安全问题

1. **Cookie 明文存储** - 应使用 macOS Keychain
2. **CSP 被禁用** - `csp: null`

---

## 🟢 改进建议

### 短期
- 移除 git 中的二进制文件
- 清理未使用的配置文件
- 统一项目命名

### 中期
- 模块化重构
- 添加单元测试
- 定义统一错误类型


