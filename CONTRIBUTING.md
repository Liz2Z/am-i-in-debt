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

1. 在 `get-cookies-script/src/` 创建新的登录脚本（如 `newplan.ts`）
2. 在 `src-tauri/src/api/` 添加 API 客户端
3. 在 `src-tauri/src/models/` 添加数据结构
4. 更新 `CodingPlan` 枚举
5. 更新文档

## 许可证

通过提交代码，你同意你的贡献将按照 MIT 许可证授权。
