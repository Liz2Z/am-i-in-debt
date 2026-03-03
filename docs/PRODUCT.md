# 产品文档

## 日志 Behavior

- **Development Mode (`bun run tauri:dev`):** Logs are enabled and visible in the console
- **Production Mode:** Logs are completely disabled and will not produce any output

## Log Levels

- `log::info!`: Informational messages about normal operations
- `log::error!`: Error messages for failures
- `log::warn!`: Warning messages for potential issues
- `log::debug!`: Debug messages for development purposes only
