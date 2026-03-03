# Development Rules

## 1. Logging instead of eprintln!/println!

**Rule:** Never use `eprintln!`, `println!`, `eprint!`, or `print!` for any output in production code. Always use the logging crate's macros instead (`log::info!`, `log::error!`, `log::warn!`, `log::debug!`).

**Reasoning:**

- Logging can be conditionally enabled/disabled based on environment
- Logs provide better structured output with timestamps and log levels
- Production builds should not produce any console output
